use log::{error, info};
use winapi::_core::{mem, ptr};
use winapi::shared::ntdef::NULL;
use winapi::um::winuser::RegisterHotKey;
use winapi::um::winuser::{GetMessageW, UnregisterHotKey, WM_HOTKEY};

pub fn register(modifier: isize, key: i32) {
    unregister();
    match unsafe { RegisterHotKey(ptr::null_mut(), 1, modifier as u32, key as u32) } {
        0 => error!("Failed to register hotkey"),
        _ => info!("Registered new hotkey"),
    };
}

fn unregister() {
    if unsafe { UnregisterHotKey(ptr::null_mut(), 1) } != 0 {
        info!("Unregistered active hotkey");
    };
}

pub fn listen<C>(mut callback: C)
where
    C: FnMut() -> (),
{
    let mut msg = unsafe { mem::zeroed() };
    loop {
        match unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } {
            0 => error!("Failed to receive message"),
            _ => {
                if WM_HOTKEY == msg.message {
                    callback();
                }
            }
        }
    }
}
