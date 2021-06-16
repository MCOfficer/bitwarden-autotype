use crate::ActiveWindowInfo;
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use log::error;
use log::info;
use parking_lot::RwLock;
use serde::Deserialize;
use serde_repr::*;
use std::ffi::{OsStr, OsString};
use std::fmt::{Display, Formatter};
use std::os::windows::process::CommandExt;
use std::process::Command;

lazy_static! {
    static ref SESSION_KEY: RwLock<Option<String>> = RwLock::new(None);
    pub static ref EMAIL: RwLock<Option<String>> = RwLock::new(None);
}

pub fn login() -> Result<()> {
    let status = status().context("Failed to get status")?;

    loop {
        let (email, password) = crate::gui::prompt_bw_login(status.user_email.clone())?;
        if password.trim().is_empty() {
            bail!("Password with len 0? This can't be right, aborting before 'bw login' stalls")
        }

        let output = if VaultStatus::Unauthenticated == status.vault_status {
            info!("Logging in...");
            call_bw(vec!["login", "--raw", email.trim(), &password])
        } else {
            info!("Already logged in, unlocking vault...");
            call_bw(vec!["unlock", "--raw", &password])
        };

        match output {
            Err(e) => {
                match e {
                    CliError::InvalidPassword => {} // loop and ask again
                    CliError::FailedToRun(io) => {
                        bail!("Calling bw failed: {:#?}", io)
                    }
                    CliError::Status(status) => {
                        bail!("Calling bw failed with status: {:#?}", status)
                    }
                }
            }
            Ok(key) => {
                let mut guard = SESSION_KEY.write();
                *guard = Some(key);
                drop(guard);

                let mut guard = EMAIL.write();
                *guard = Some(email);
                drop(guard);

                return Ok(());
            }
        }
        info!("Acquired session key");
    }
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum VaultStatus {
    Unlocked,
    Locked,
    Unauthenticated,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    server_url: String,
    last_sync: Option<DateTime<Utc>>,
    user_email: Option<String>,
    user_id: Option<String>,
    #[serde(rename = "status")]
    vault_status: VaultStatus,
}

pub fn status() -> Result<Status> {
    let stdout = call_bw(vec!["status"])?;
    let status: Status = serde_json::from_str(&stdout).context("Failed to serialize status")?;
    Ok(status)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub name: String,
    pub notes: Option<String>,
    pub login: Option<Login>,
}

impl LoginItem {
    pub fn autotype_pattern(&self) -> Option<String> {
        let indicator = "Autotype: ";
        let mut lines = self.notes.as_ref()?.lines();
        lines.find_map(|l| l.strip_prefix(indicator).map(|s| s.to_string()))
    }

    pub fn totp(&self) -> Result<String> {
        Ok(call_bw(vec!["get", "totp", &self.id])?)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ItemType {
    Login = 1,
    SecureNote = 2,
    Card = 3,
    Identity = 4,
}

pub fn list_logins(info: ActiveWindowInfo) -> Result<Vec<LoginItem>> {
    let stdout = call_bw(vec!["list", "items", "--url", &info.title])?;
    let mut logins: Vec<LoginItem> = serde_json::from_str(&stdout)?;

    let stdout = call_bw(vec!["list", "items", "--url", &info.executable])?;
    let executable_logins: Vec<LoginItem> = serde_json::from_str(&stdout)?;
    logins.extend(executable_logins);
    Ok(logins)
}

pub fn sync() {
    info!("Syncing");
    if let Err(e) = call_bw(vec!["sync"]) {
        error!("Failed to perform sync: {:?}", e);
    }
}

#[derive(Debug)]
enum CliError {
    InvalidPassword,
    Status(std::process::ExitStatus),
    FailedToRun(std::io::Error),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CliError {}

fn call_bw<A>(args: Vec<A>) -> std::result::Result<String, CliError>
where
    A: Into<OsString> + AsRef<OsStr>,
{
    let output = Command::new("bw")
        .args(&args)
        .env(
            "BW_SESSION",
            SESSION_KEY.read().as_ref().unwrap_or(&"".to_string()), // Passing an empty string will make bitwarden ignore it
        )
        .creation_flags(0x08000000) //CREATE_NO_WINDOW
        .output()
        .map_err(CliError::FailedToRun)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        if stderr.contains("Invalid master password") {
            return Err(CliError::InvalidPassword);
        }

        let subcmd: OsString = args.get(0).unwrap().into();
        let status_code = output
            .status
            .code()
            .map_or("(No exit code)".into(), |u| u.to_string());
        error!(
            "'bw {}' returned non-zero exit code {}",
            subcmd.to_string_lossy(),
            status_code
        );
        error!("STDERR WAS:\n{}", stderr);
        error!("STDOUT WAS:\n{}", stdout);

        return Err(CliError::Status(output.status));
    }

    Ok(stdout.into())
}
