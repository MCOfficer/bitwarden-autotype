mod bw_cli;
mod gui;
mod hotkeys;

use anyhow::Result;
use chrono;
use fern;
use log::info;
use log::LevelFilter;
use std::time::Duration;
use win_key_codes::VK_A;
use winapi::um::winuser::{MOD_ALT, MOD_CONTROL};

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

fn main() {
    setup_logger();
    bw_cli::login().unwrap();
    listen_to_hotkeys().unwrap();
}
