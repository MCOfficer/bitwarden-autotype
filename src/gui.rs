use anyhow::Result;

use fltk::app::{set_focus, App};
use fltk::button::Button;
use fltk::enums::{Key, Shortcut};
use fltk::image::PngImage;
use fltk::input::{Input, SecretInput};
use fltk::prelude::*;
use fltk::window::Window;

pub fn prompt_login(bitwarden_email: Option<String>) -> Result<(String, String)> {
    let app = App::default();
    let mut window = Window::new(100, 100, 400, 150, "Log into Bitwarden");
    let icon = PngImage::from_data(include_bytes!("../assets/icon.png")).unwrap();
    window.set_icon(Some(icon));

    let email = Input::new(80, 20, 300, 30, "E-Mail");
    let password = SecretInput::new(80, 70, 300, 30, "Password");
    let mut submit = Button::new(160, 110, 80, 30, "Submit");

    window.end();
    window.show();

    if let Some(e) = bitwarden_email {
        email.set_value(&e);
        set_focus(&password);
    }
    submit.set_shortcut(Shortcut::from_key(Key::Enter));
    submit.set_callback(move |_| app.quit());

    app.run().unwrap();
    Ok((email.value(), password.value()))
}
