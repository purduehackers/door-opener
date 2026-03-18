use macroquad::prelude::*;

use crate::gui::MessageOpacities;
use crate::gui::colors::{BLACK_BG, YELLOW_ACCENT};
use crate::gui::constants::{OPACITY_MAX, OPACITY_MIN, SCREEN_WIDTH, TEXT_MARGIN};
use crate::gui::font_engine::{Point, draw_text};

#[allow(clippy::cast_possible_truncation)]
fn opacity_to_u8(opacity: f32) -> u8 {
    let clamped = opacity.clamp(OPACITY_MIN, OPACITY_MAX).round() as i32;
    u8::try_from(clamped).unwrap_or(0)
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

fn draw_welcome_window(opacity: u8, font: &Font) {
    draw_rectangle(0.0, 164.0, SCREEN_WIDTH, 392.0, BLACK_BG(opacity));
    draw_rectangle(0.0, 164.0, SCREEN_WIDTH, 4.0, YELLOW_ACCENT(opacity));
    draw_rectangle(0.0, 552.0, SCREEN_WIDTH, 4.0, YELLOW_ACCENT(opacity));

    let _ = draw_text(
        "Welcome to Hack Night",
        Point::new(TEXT_MARGIN, 203.0),
        SCREEN_WIDTH - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        96,
        1.0,
    );
    let _ = draw_text(
        "Scan your passport or dial the phone bell",
        Point::new(TEXT_MARGIN, 422.0),
        SCREEN_WIDTH - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        48,
        1.0,
    );
}

fn draw_accepted_window(opacity: u8, font: &Font) {
    draw_rectangle(0.0, 212.0, SCREEN_WIDTH, 296.0, BLACK_BG(opacity));
    draw_rectangle(0.0, 212.0, SCREEN_WIDTH, 4.0, YELLOW_ACCENT(opacity));
    draw_rectangle(0.0, 504.0, SCREEN_WIDTH, 4.0, YELLOW_ACCENT(opacity));

    let _ = draw_text(
        "Welcome back!",
        Point::new(TEXT_MARGIN, 251.0),
        SCREEN_WIDTH - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        96,
        1.0,
    );
    let _ = draw_text(
        "Please be mindful of the door opening",
        Point::new(TEXT_MARGIN, 374.0),
        SCREEN_WIDTH - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        48,
        1.0,
    );
}

fn draw_error_window(opacity: u8, font: &Font, title: &str, subtitle: &str) {
    draw_rectangle(0.0, 140.0, SCREEN_WIDTH, 440.0, BLACK_BG(opacity));
    draw_rectangle(0.0, 140.0, SCREEN_WIDTH, 4.0, YELLOW_ACCENT(opacity));
    draw_rectangle(0.0, 576.0, SCREEN_WIDTH, 4.0, YELLOW_ACCENT(opacity));

    let _ = draw_text(
        title,
        Point::new(TEXT_MARGIN, 179.0),
        SCREEN_WIDTH - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        96,
        1.0,
    );
    let _ = draw_text(
        subtitle,
        Point::new(TEXT_MARGIN, 398.0),
        SCREEN_WIDTH - TEXT_MARGIN,
        YELLOW_ACCENT(opacity),
        font,
        48,
        1.0,
    );
}
