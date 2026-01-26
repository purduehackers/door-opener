use macroquad::prelude::*;

const YELLOW_ACCENT: (u8, u8, u8) = (251, 203, 59);
const BLACK_BG: (u8, u8, u8) = (10, 10, 10);
const WHITE: (u8, u8, u8) = (255, 255, 255);

pub fn yellow_accent(opacity: u8) -> Color {
    Color::from_rgba(YELLOW_ACCENT.0, YELLOW_ACCENT.1, YELLOW_ACCENT.2, opacity)
}

pub fn black_bg(opacity: u8) -> Color {
    Color::from_rgba(BLACK_BG.0, BLACK_BG.1, BLACK_BG.2, opacity)
}

pub fn white(opacity: u8) -> Color {
    Color::from_rgba(WHITE.0, WHITE.1, WHITE.2, opacity)
}
