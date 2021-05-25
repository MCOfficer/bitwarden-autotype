mod bw_cli;
mod gui;

use chrono;
use fern;
use log::LevelFilter;
use std::thread::sleep;
use std::time::Duration;

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

fn register_hotkeys() {
    // TODO
}

fn main() {
    setup_logger();
    bw_cli::login().unwrap();
    register_hotkeys();
    loop {
        sleep(Duration::from_millis(50));
    }
}
