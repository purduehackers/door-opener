use macroquad::prelude::*;

const YELLOW_ACCENT: (u8, u8, u8) = (251, 203, 59);
const BLACK_BG: (u8, u8, u8) = (10, 10, 10);
const WHITE: (u8, u8, u8) = (255, 255, 255);
// Suffixed with CL to distin
pub const GREEN_CL: Color = Color::from_hex(0x22c55e);
pub const RED_CL: Color = Color::from_hex(0xef4444);

pub const fn yellow_accent(opacity: u8) -> Color {
    Color::from_rgba(YELLOW_ACCENT.0, YELLOW_ACCENT.1, YELLOW_ACCENT.2, opacity)
}

pub const fn black_bg(opacity: u8) -> Color {
    Color::from_rgba(BLACK_BG.0, BLACK_BG.1, BLACK_BG.2, opacity)
}

pub const fn white(opacity: u8) -> Color {
    Color::from_rgba(WHITE.0, WHITE.1, WHITE.2, opacity)
}
