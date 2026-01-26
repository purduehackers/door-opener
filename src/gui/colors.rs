use macroquad::prelude::*;

pub const YELLOW_ACCENT: fn(u8) -> Color = |opacity| Color::from_rgba(251, 203, 59, opacity);

pub const BLACK_BG: fn(u8) -> Color = |opacity| Color::from_rgba(10, 10, 10, opacity);

// Suffixed with CL to distinguish from macroquad built-ins
pub const WHITE_CL: fn(u8) -> Color = |opacity| Color::from_rgba(255, 255, 255, opacity);
pub const GREEN_CL: Color = Color::from_hex(0x22c55e);
pub const RED_CL: Color = Color::from_hex(0xef4444);
