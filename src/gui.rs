extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use self::nwg::EventData;
use anyhow::{Context, Result};
use nwd::NwgUi;
use nwg::NativeUi;

#[derive(Default, NwgUi)]
pub struct LoginWindow {
    #[nwg_control(size: (300, 135), position: (300, 300), title: "Log into Bitwarden", flags: "WINDOW|VISIBLE")]
    window: nwg::Window,

    #[nwg_control(size: (280, 35), position: (10, 10), focus: true, placeholder_text: Some("E-Mail"), flags: "TAB_STOP|AUTO_SCROLL|VISIBLE")]
    email_field: nwg::TextInput,

    #[nwg_control(size: (280, 35), position: (10, 50), placeholder_text: Some("Password"),  password: Some('*'))]
    password_field: nwg::TextInput,

    #[nwg_control(text: "Login", position: (10, 90), )]
    #[nwg_events(OnButtonClick: [LoginWindow::submit])]
    submit_btn: nwg::Button,
}

impl LoginWindow {
    fn submit(&self) {
        self.window.close();
        nwg::stop_thread_dispatch();
    }
}

pub fn prompt_login(email: Option<String>) -> Result<(String, String)> {
    nwg::init().context("Failed to initialize NWG")?;
    let app = LoginWindow::build_ui(Default::default()).context("Failed to build window")?;

    if let Some(email) = email {
        app.email_field.set_text(&email);
        app.password_field.set_focus();
    }

    nwg::dispatch_thread_events();
    Ok((app.email_field.text(), app.password_field.text()))
}
