mod bw_cli;
mod gui;
mod hotkeys;
mod tray;

use crate::bw_cli::LoginItem;
use chrono;
use fern;
use log::LevelFilter;
use log::{error, info};
use std::collections::HashMap;
use std::panic::catch_unwind;
use std::time::Duration;
use strfmt::Format;
use win_key_codes::VK_A;
use winapi::um::winuser::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, MOD_ALT, MOD_CONTROL,
};

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
    hotkeys::listen(|| handle_hotkey());
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
    let interval = std::time::Duration::from_millis(20);

    let mut vars = HashMap::new();
    vars.insert(
        "USERNAME".into(),
        item.login
            .as_ref()
            .map(|l| l.username.clone())
            .flatten()
            .unwrap_or("".to_string()),
    );
    vars.insert(
        "PASSWORD".into(),
        item.login
            .as_ref()
            .map(|l| l.password.clone())
            .flatten()
            .unwrap_or("".to_string()),
    );
    vars.insert("TAB".into(), "\t".into());
    vars.insert("ENTER".into(), "\n".into());

    let pattern = "{USERNAME}{TAB}{PASSWORD}{ENTER}";
    match pattern.format(&vars) {
        Ok(to_type) => {
            for char in to_type.chars() {
                send_char(char);
                std::thread::sleep(interval);
            }
        }
        Err(e) => {
            error!("Formatting error! {:?}", e)
        }
    };
}

fn send_char(c: char) {
    if let Err(_) = catch_unwind(|| match c {
        '\t' => winput::send(winput::Vk::Tab),
        '\n' => winput::send(winput::Vk::Enter),
        _ => winput::send(c),
    }) {
        error!("Failed to send keystroke for character");
    }
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

fn main() {
    setup_logger();
    bw_cli::login().unwrap();

    std::thread::spawn(|| listen_to_hotkeys());

    std::thread::spawn(|| {
        info!("Starting Syncing thread");
        loop {
            std::thread::sleep(Duration::from_secs(60 * 5));
            bw_cli::sync();
        }
    });

    let email = bw_cli::EMAIL.read().clone().unwrap_or("(unknown)".into());
    tray::main(email);
}
