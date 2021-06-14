#![windows_subsystem = "windows"]

mod bw_cli;
mod gui;
mod hotkeys;
mod tray;
mod typing;

use crate::bw_cli::LoginItem;

use crate::typing::send_raw_string;
use argh::FromArgs;
use log::LevelFilter;
use log::{error, info};
use std::io::{stdin, BufRead};
use std::process::exit;
use std::time::Duration;
use win_key_codes::VK_A;
use winapi::um::winuser::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, MOD_ALT, MOD_CONTROL,
};

static DEFAULT_PATTERN: &str = "{USERNAME}{TAB}{PASSWORD}{ENTER}";

fn setup_logger() {
    fern::Dispatch::new()
        .level(LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {} {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("bitwarden-autotype.log").unwrap())
        .apply()
        .unwrap();
}

fn listen_to_hotkeys() {
    hotkeys::register(MOD_ALT | MOD_CONTROL, VK_A);
    hotkeys::listen(handle_hotkey);
}

fn handle_hotkey() {
    info!("Received hotkey event");
    let window_title = active_window();
    match bw_cli::list_logins(&window_title) {
        Ok(logins) => {
            match logins.len() {
                0 => error!("Bitwarden returned no matching logins"),
                1 => autotype(logins.get(0).unwrap()),
                _ => match gui::login_choice(logins) {
                    Ok(item) => autotype(&item),
                    Err(e) => error!("Failed to ask the user to choose a login: {:?}", e),
                },
            };
        }
        Err(e) => error!("Failed to get logins: {:?}", e),
    };
}

fn autotype(item: &LoginItem) {
    info!("Autotype for {}", item.name);

    let mut pattern = item
        .autotype_pattern()
        .unwrap_or(DEFAULT_PATTERN.to_string());

    pattern = pattern.replace(
        "{USERNAME}",
        &item
            .login
            .as_ref()
            .map(|l| l.username.clone())
            .flatten()
            .unwrap_or_else(|| "".to_string()),
    );
    pattern = pattern.replace(
        "{PASSWORD}",
        &item
            .login
            .as_ref()
            .map(|l| l.password.clone())
            .flatten()
            .unwrap_or_else(|| "".to_string()),
    );
    send_raw_string(pattern);
}

fn active_window() -> String {
    let handle = unsafe { GetForegroundWindow() }; // First, get the window handle
    let title_len = unsafe { GetWindowTextLengthW(handle) } + 1; // Get the title length (+1 to be sure)

    let mut buffer: Vec<u16> = Vec::with_capacity(title_len as usize); // Create a buffer that windows can fill
    let read_len = unsafe { GetWindowTextW(handle, buffer.as_mut_ptr(), title_len) }; // Tell windows to fill the buffer

    // Tell the buffer how much has been read into it, lest it still thinks it's empty, resulting in an empty string
    unsafe { buffer.set_len(read_len as usize) };
    String::from_utf16_lossy(buffer.as_slice())
}

#[derive(FromArgs)]
/// Yes, this thing has a CLI.
struct BitwardenAutotype {
    /// run in autotype-server mode. Since this autotypes anything it receives on stdin,
    /// calling this from a terminal yourself results in a very nice example of an infinite feedback loop.
    #[argh(switch)]
    server: bool,
}

fn main() {
    let opts: BitwardenAutotype = argh::from_env();
    setup_logger();

    if opts.server {
        run_as_server();
    }

    bw_cli::login().unwrap();

    std::thread::spawn(listen_to_hotkeys);

    std::thread::spawn(|| {
        info!("Starting Syncing thread");
        loop {
            bw_cli::sync();
            std::thread::sleep(Duration::from_secs(60 * 5));
        }
    });

    let email = bw_cli::EMAIL
        .read()
        .clone()
        .unwrap_or_else(|| "(unknown)".into());
    tray::main(email);
}

fn run_as_server() {
    for res in stdin().lock().lines() {
        match res {
            Ok(line) => typing::send_serialized_cmd(line),
            Err(e) => error!("Failed to read line from stdin: {}", e),
        }
    }
    exit(0);
}
