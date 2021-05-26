mod bw_cli;
mod gui;
mod hotkeys;

use anyhow::Result;
use chrono;
use fern;
use log::info;
use log::LevelFilter;
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

fn listen_to_hotkeys() -> Result<()> {
    hotkeys::register(MOD_ALT | MOD_CONTROL, VK_A);
    hotkeys::listen(|| handle_hotkey());
    Ok(())
}

fn handle_hotkey() {
    info!("STUB handle hotkey");
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
    listen_to_hotkeys().unwrap();
}
