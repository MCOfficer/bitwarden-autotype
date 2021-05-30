use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use duct::cmd;
use lazy_static::lazy_static;
use log::error;
use log::info;
use parking_lot::RwLock;
use serde::Deserialize;
use serde_repr::*;
use std::ffi::{OsStr, OsString};

lazy_static! {
    static ref SESSION_KEY: RwLock<Option<String>> = RwLock::new(None);
    pub static ref EMAIL: RwLock<Option<String>> = RwLock::new(None);
}

pub fn login() -> Result<()> {
    let status = status().context("Failed to get status")?;

    let (email, password) = crate::gui::prompt_login(status.user_email)?;
    if password.trim().len() == 0 {
        bail!("Password with len 0? This can't be right, aborting before 'bw login' stalls")
    }

    let session = if VaultStatus::UNAUTHENTICATED == status.vault_status {
        info!("Logging in...");
        call_bw(vec!["login", "--raw", email.trim(), &password])?
    } else {
        info!("Already logged in, unlocking vault...");
        call_bw(vec!["unlock", "--raw", &password])?
    };
    info!("Acquired session key");

    let mut guard = SESSION_KEY.write();
    *guard = Some(session);
    drop(guard);

    let mut guard = EMAIL.write();
    *guard = Some(email);
    drop(guard);

    Ok(())
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum VaultStatus {
    UNLOCKED,
    LOCKED,
    UNAUTHENTICATED,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub name: String,
    pub notes: Option<String>,
    pub login: Option<Login>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum ItemType {
    Login = 1,
    SecureNote = 2,
    Card = 3,
    Identity = 4,
}

pub fn list_logins(url: &str) -> Result<Vec<LoginItem>> {
    let stdout = call_bw(vec!["list", "items", "--url", url])?;
    let logins: Vec<LoginItem> = serde_json::from_str(&stdout)?;
    Ok(logins)
}

pub fn sync() {
    info!("Syncing");
    if let Err(e) = call_bw(vec!["sync"]) {
        error!("Failed to perform sync: {:?}", e);
    }
}

fn call_bw<A>(args: Vec<A>) -> Result<String>
where
    A: Into<OsString> + AsRef<OsStr>,
{
    let cmd = cmd("bw", &args)
        .stderr_capture()
        .stdout_capture()
        .env(
            "BW_SESSION",
            SESSION_KEY.read().as_ref().unwrap_or(&"".to_string()), // Passing an empty string will make bitwarden ignore it
        )
        .unchecked();

    let output = cmd.run().context("Error running command")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
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

        return Err(anyhow::format_err!("Error calling bitwarden cli"));
    }

    Ok(stdout.into())
}
