use anyhow::Result;

use crate::bw_cli::LoginItem;
use fltk::app::{set_focus, App};
use fltk::button::Button;
use fltk::enums::{Key, Shortcut};
use fltk::image::PngImage;
use fltk::input::{Input, SecretInput};
use fltk::prelude::*;
use fltk::table::{Table, TableContext};
use fltk::window::Window;
use fltk::{draw, enums};
use lazy_static::lazy_static;

lazy_static! {
    static ref ICON: PngImage = PngImage::from_data(include_bytes!("../assets/icon.png")).unwrap();
}

pub fn prompt_bw_login(bitwarden_email: Option<String>) -> Result<(String, String)> {
    let app = App::default();
    let mut window = Window::new(100, 100, 400, 150, "Log into Bitwarden");
    window.set_icon(Some(ICON.clone()));

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
    // Required so we can spawn windows in separate threads (like the hotkey thread).
    // As of now, no two GUIs can run concurrently, so this will not cause issues.
    fltk::app::unlock();
    Ok((email.value(), password.value()))
}

pub fn login_choice(items: Vec<LoginItem>) -> Result<LoginItem> {
    let app = App::default();
    let mut window = Window::new(100, 100, 400, 150, "Choose a Login");
    window.set_icon(Some(ICON.clone()));

    let mut table = Table::new(20, 10, 400 - 40, 150 - 20 - 50, "");
    table.set_rows(items.len() as i32);
    table.set_cols(2);
    table.set_col_width_all(table.width() / 2 - 1);
    table.set_selection(0, 0, 0, 0);
    table.end();

    let mut submit = Button::new(160, 110, 80, 30, "Shoot!");

    window.end();
    window.show();

    let closure_items = items.clone();
    table.draw_cell(move |t, ctx, row, col, x, y, w, h| {
        if ctx == TableContext::Cell {
            let item: &LoginItem = closure_items.get(row as usize).unwrap();
            let data = match col {
                0 => item.name.clone(),
                1 => item
                    .login
                    .clone()
                    .map(|l| l.username)
                    .flatten()
                    .unwrap_or_else(|| "".into()),
                _ => "".into(),
            };
            draw_data(&data, x, y, w, h, t.is_selected(row, col))
        }
    });

    submit.set_shortcut(Shortcut::from_key(Key::Enter));
    submit.set_callback(move |_| app.quit());

    app.run().unwrap();
    let (i, _, _, _) = table.get_selection();
    // Required so we can spawn windows in separate threads (like the hotkey thread).
    // As of now, no two GUIs can run concurrently, so this will not cause issues.
    fltk::app::unlock();
    Ok(items.get(i as usize).unwrap().clone())
}

// The selected flag sets the color of the cell to a grayish color, otherwise white
fn draw_data(txt: &str, x: i32, y: i32, w: i32, h: i32, selected: bool) {
    draw::push_clip(x, y, w, h);
    if selected {
        draw::set_draw_color(enums::Color::from_u32(0x00D3_D3D3));
    } else {
        draw::set_draw_color(enums::Color::White);
    }
    draw::draw_rectf(x, y, w, h);
    draw::set_draw_color(enums::Color::Gray0);
    draw::draw_text2(txt, x, y, w, h, enums::Align::Center);
    draw::draw_rect(x, y, w, h);
    draw::pop_clip();
}
