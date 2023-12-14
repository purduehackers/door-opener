use std::sync::mpsc::Receiver;

use macroquad::prelude::*;

use self::passport::draw_passport;

mod background;
mod passport;
mod svg;

const SEGOE_UI_FONT: &[u8] = include_bytes!("./SegoeUI.ttf");

pub fn gui_entry(nfc_messages: Receiver::<i32>) {
    macroquad::Window::from_config(Conf {
        window_title: "Door Opener".to_owned(),
        fullscreen: false,
        window_width: 720,
        window_height: 720,
        window_resizable: false,
        sample_count: 4,
        ..Default::default()
    }, gui_main(nfc_messages))
}

async fn gui_main(nfc_messages: Receiver::<i32>) {
    let mut current_nfc_status: i32 = 0;

    let segoe_ui = load_ttf_font_from_bytes(SEGOE_UI_FONT)
        .unwrap();

    let background_data = background::initialise_background().await;

    let passport_data = passport::initialise_passport().await;

    loop {
        let next_message = nfc_messages.try_recv();

        if next_message.is_ok() {
            current_nfc_status = next_message.unwrap();
        }

        clear_background(Color::from_hex(0x0a0a0a));

        background::draw_background(&background_data);

        draw_text_window(
            "Welcome to\nHack Night", 
            "Scan your passport to\nstart", 
            false, 
            1.0, 
            &segoe_ui
        );

        draw_passport(20.0, 20.0, 0, &passport_data);

        next_frame().await
    }
}

fn draw_text_window(title: &str, description: &str, show_qr: bool, opacity: f32, font: &Font) {
    draw_text_ex(title, 20.0, 200.0, TextParams {
        font_size: 96,
        color: Color::from_hex(0xfbcb3b),
        font: Some(&font),
        ..Default::default()
    });
    draw_text_ex(description, 20.0, 300.0, TextParams {
        font_size: 48,
        color: Color::from_hex(0xfbcb3b),
        font: Some(&font),
        ..Default::default()
    });
}