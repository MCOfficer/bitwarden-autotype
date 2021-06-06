use log::{info, warn};
use std::process::exit;
use trayicon::{Icon, MenuBuilder, MenuItem, TrayIconBuilder};
use winapi::_core::mem::MaybeUninit;
use winapi::um::winuser;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Events {
    Exit,
    Dummy,
    SyncNow,
}

fn info_item(name: &str) -> MenuItem<Events> {
    MenuItem::Item {
        id: Events::Dummy,
        name: name.to_string(),
        disabled: true,
        icon: None,
    }
}

pub fn main(username: String) {
    let icon_bytes = include_bytes!("../assets/icon.ico");
    let icon = Icon::from_buffer(icon_bytes, None, None).unwrap();
    let (s, r) = std::sync::mpsc::channel::<Events>();

    let _tray_icon = TrayIconBuilder::new()
        .icon(icon)
        .sender(s)
        .tooltip("Bitwarden Autotype")
        .menu(
            MenuBuilder::new()
                .with(info_item("Bitwarden Autotype is running"))
                .separator()
                .with(info_item(&format!("Logged in as {}", username)))
                .with(info_item("Hotkey: Ctrl-Alt-A"))
                .separator()
                .item("Sync Vault", Events::SyncNow)
                .separator()
                .item("Exit", Events::Exit),
        )
        .build()
        .unwrap();

    std::thread::spawn(move || {
        r.iter().for_each(|m| match m {
            Events::Exit => {
                info!("Shutting down");
                exit(0);
            }
            Events::SyncNow => {
                info!("Sync requested");
                crate::bw_cli::sync()
            }
            e => {
                println!("{:?}", e);
            }
        })
    });

    loop {
        unsafe {
            let mut msg = MaybeUninit::uninit();
            let bret = winuser::GetMessageW(msg.as_mut_ptr(), 0 as _, 0, 0);
            if bret > 0 {
                winuser::TranslateMessage(msg.as_ptr());
                winuser::DispatchMessageW(msg.as_ptr());
            } else {
                warn!("Failed to receive message");
            }
        }
    }
}
