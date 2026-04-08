use macroquad::prelude::*;

use crate::gui::MessageOpacities;
use crate::gui::colors::{BLACK_BG, YELLOW_ACCENT};
use crate::gui::constants::{OPACITY_MAX, OPACITY_MIN, TEXT_MARGIN};
use crate::gui::font_engine::{Point, draw_text};

#[allow(clippy::cast_possible_truncation)]
fn opacity_to_u8(opacity: f32) -> u8 {
    let clamped = opacity.clamp(OPACITY_MIN, OPACITY_MAX).round() as i32;
    u8::try_from(clamped).unwrap_or(0)
}

fn get_heading_text_size() -> u16 {
    if screen_height() < 720.0 { 72 } else { 96 }
}

fn get_desc_text_size() -> u16 {
    if screen_height() < 720.0 { 32 } else { 48 }
}

pub fn draw_message_windows(opacities: &MessageOpacities, font: &Font) {
    draw_welcome_window(opacity_to_u8(opacities.welcome), font);
    draw_accepted_window(opacity_to_u8(opacities.accepted), font);
    draw_error_window(
        opacity_to_u8(opacities.rejected),
        font,
        "Invalid Passport!",
        "Please try to scan your passport again!",
    );
    draw_error_window(
        opacity_to_u8(opacities.net_error),
        font,
        "Something went wrong!",
        "We're having connectivity issues at the moment. Please try again.",
    );
    draw_error_window(
        opacity_to_u8(opacities.nfc_error),
        font,
        "NFC read error!",
        "Please take away your passport, then hold it still during the scan!",
    );
    draw_error_window(
        opacity_to_u8(opacities.doorhw_not_ready_error),
        font,
        "Button pusher not ready yet!",
        "Try again after a minute or contact an organizer.",
    );
}

fn draw_message_box(opacity: u8, margin_percentage: f32, content_percentage: f32) {
    let height = screen_height();
    draw_rectangle(
        0.0,
        height * margin_percentage,
        screen_width(),
        height * content_percentage,
        BLACK_BG(opacity),
    );
    draw_rectangle(
        0.0,
        height * margin_percentage,
        screen_width(),
        4.0,
        YELLOW_ACCENT(opacity),
    );
    draw_rectangle(
        0.0,
        height * (margin_percentage + content_percentage),
        screen_width(),
        4.0,
        YELLOW_ACCENT(opacity),
    );
}

fn draw_welcome_window(opacity: u8, font: &Font) {
    let top_bottom_margin_percentage = 0.23;
    let main_content_percentage = 1.0 - top_bottom_margin_percentage * 2.0;
    draw_message_box(
        opacity,
        top_bottom_margin_percentage,
        main_content_percentage,
    );

    let height = screen_height();
    let heading_start_percentage = if screen_height() < 720.0 { 0.30 } else { 0.28 };
    let _ = draw_text(
        "Welcome to Hack Night",
        Point::new(TEXT_MARGIN, height * heading_start_percentage),
        screen_width() - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        get_heading_text_size(),
        1.0,
    );

    let desc_start_percentage = 0.58;
    let _ = draw_text(
        "Scan your passport or dial the phone bell",
        Point::new(TEXT_MARGIN, height * desc_start_percentage),
        screen_width() - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        get_desc_text_size(),
        1.0,
    );
}

fn draw_accepted_window(opacity: u8, font: &Font) {
    let top_bottom_margin_percentage = 0.29;
    let main_content_percentage = 1.0 - top_bottom_margin_percentage * 2.0;
    draw_message_box(
        opacity,
        top_bottom_margin_percentage,
        main_content_percentage,
    );

    let height = screen_height();
    let heading_start_percentage = 0.348;
    let _ = draw_text(
        "Welcome back!",
        Point::new(TEXT_MARGIN, height * heading_start_percentage),
        screen_width() - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        get_heading_text_size(),
        1.0,
    );

    let desc_start_percentage = 0.52;
    let _ = draw_text(
        "Please be mindful of the door opening",
        Point::new(TEXT_MARGIN, height * desc_start_percentage),
        screen_width() - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        get_desc_text_size(),
        1.0,
    );
}

fn draw_error_window(opacity: u8, font: &Font, title: &str, subtitle: &str) {
    let top_bottom_margin_percentage = 0.19;
    let main_content_percentage = 1.0 - top_bottom_margin_percentage * 2.0;
    draw_message_box(
        opacity,
        top_bottom_margin_percentage,
        main_content_percentage,
    );

    let height = screen_height();
    let heading_start_percentage = 0.248;
    let _ = draw_text(
        title,
        Point::new(TEXT_MARGIN, height * heading_start_percentage),
        screen_width() - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        get_heading_text_size(),
        1.0,
    );

    let desc_start_percentage = 0.55;
    let _ = draw_text(
        subtitle,
        Point::new(TEXT_MARGIN, height * desc_start_percentage),
        screen_width() - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        get_desc_text_size(),
        1.0,
    );
}
