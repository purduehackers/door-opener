use macroquad::prelude::*;

use super::svg;

const PASSPORT_EMBLEM: &[u8] = include_bytes!("../assets/passport-emblem.svg");
const LOADING_SPINNER: &[u8] = include_bytes!("../assets/loading-spinner.svg");

pub struct PassportData {
    logo_texture: Texture2D,
    loading_spinner_texture: Texture2D,
    current_spinner_colour: Color,
    current_spinner_cutout_opacity: f32,
    current_x: f32,
    current_y: f32,
    last_state: i32,
    current_animation_time: f32,
    last_final_x: f32,
    last_final_y: f32,
}

pub async fn initialise_passport() -> PassportData {
    let logo_texture = svg::svg_to_texture(
        String::from_utf8(PASSPORT_EMBLEM.to_vec())
            .unwrap()
            .as_str(),
    );
    let loading_spinner_texture = svg::svg_to_texture(
        String::from_utf8(LOADING_SPINNER.to_vec())
            .unwrap()
            .as_str(),
    );

    return PassportData {
        logo_texture: logo_texture,
        loading_spinner_texture: loading_spinner_texture,
        current_spinner_colour: Color::from_hex(0xfbcb3b),
        current_spinner_cutout_opacity: 0.0,
        current_x: 0.0,
        current_y: 0.0,
        last_state: 0,
        current_animation_time: 0.0,
        last_final_x: 0.0,
        last_final_y: 0.0,
    };
}

pub fn draw_passport(x: f32, y: f32, state: i32, passport_data: &mut PassportData) {
    let delta_time: f32 = get_frame_time();
    let loading_spinner_angle = (get_time() * 3.0) as f32;

    if state != passport_data.last_state {
        passport_data.last_state = state;
        passport_data.current_animation_time = 0.0;
        passport_data.last_final_x = passport_data.current_x;
        passport_data.last_final_y = passport_data.current_y;
    }

    match state {
        0 => {
            passport_data.current_x = x;
            passport_data.current_y = y;
        }
        1 => {
            let linear_x = f32::clamp(passport_data.current_animation_time, 0.0, 1.0);
            let curved_x: f32 =
                -2.0 * (linear_x * linear_x * linear_x) + 3.0 * (linear_x * linear_x);

            passport_data.current_x = super::float32_lerp(passport_data.last_final_x, x, curved_x);
            passport_data.current_y = super::float32_lerp(passport_data.last_final_y, y, curved_x);
        }
        _default => {
            let linear_x = passport_data.current_animation_time - 1.0;
            let mut curved_x: f32 = 0.0;

            if linear_x >= 0.0 {
                curved_x = -2.0 * (linear_x * linear_x * linear_x) + 3.0 * (linear_x * linear_x);
            }

            passport_data.current_x = super::float32_lerp(passport_data.last_final_x, x, curved_x);
            passport_data.current_y = super::float32_lerp(passport_data.last_final_y, y, curved_x);
        }
    }

    passport_data.current_animation_time =
        f32::clamp(passport_data.current_animation_time + delta_time, 0.0, 2.0);

    draw_rectangle(
        passport_data.current_x - 360.0,
        passport_data.current_y - 360.0,
        720.0,
        720.0,
        Color::from_hex(0xfbcb3b),
    );
    draw_rectangle(
        passport_data.current_x - 350.0,
        passport_data.current_y - 350.0,
        700.0,
        700.0,
        Color::from_rgba(10, 10, 10, 255),
    );

    passport_data.current_spinner_cutout_opacity = super::float32_lerp(
        passport_data.current_spinner_cutout_opacity,
        match state {
            0 => 1.0,
            1 => 0.0,
            2 => 1.0,
            3 => 1.0,
            _default => 1.0,
        },
        delta_time * 10.0,
    );
    passport_data.current_spinner_colour = super::colour_lerp(
        passport_data.current_spinner_colour,
        match state {
            0 => Color::from_hex(0xfbcb3b),
            1 => Color::from_hex(0xfbcb3b),
            2 => Color::from_hex(0x22c55e),
            3 => Color::from_hex(0xef4444),
            _default => Color::from_hex(0xfbcb3b),
        },
        delta_time * 10.0,
    );

    draw_texture(
        &passport_data.logo_texture,
        passport_data.current_x - (passport_data.logo_texture.width() / 2.0),
        passport_data.current_y - (passport_data.logo_texture.height() / 2.0),
        Color::from_hex(0xfbcb3b),
    );

    let spinner_center_x =
        passport_data.current_x - (passport_data.loading_spinner_texture.width() / 2.0);
    let spinner_center_y =
        passport_data.current_y - (passport_data.loading_spinner_texture.height() / 2.0);

    draw_texture_ex(
        &passport_data.loading_spinner_texture,
        spinner_center_x,
        spinner_center_y,
        Color {
            r: passport_data.current_spinner_colour.r,
            g: passport_data.current_spinner_colour.g,
            b: passport_data.current_spinner_colour.b,
            a: passport_data.current_spinner_cutout_opacity,
        },
        DrawTextureParams {
            dest_size: Option::None,
            source: Option::None,
            rotation: loading_spinner_angle,
            flip_x: true,
            flip_y: true,
            pivot: Option::None,
        },
    );

    draw_texture_ex(
        &passport_data.loading_spinner_texture,
        spinner_center_x,
        spinner_center_y,
        passport_data.current_spinner_colour,
        DrawTextureParams {
            dest_size: Option::None,
            source: Option::None,
            rotation: loading_spinner_angle,
            flip_x: false,
            flip_y: false,
            pivot: Option::None,
        },
    );
}
