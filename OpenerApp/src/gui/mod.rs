pub mod background;
pub mod passport;
pub mod svg;
pub mod font_engine;

use std::sync::mpsc::Receiver;

use macroquad::prelude::*;

use self::{passport::draw_passport, font_engine::draw_text};
use crate::timedvariable::TimedVariable;

// use crate::hardware::led::LEDController;

const SEGOE_UI_FONT: &[u8] = include_bytes!("./assets/SegoeUI.ttf");
const DOORBELL_QR: &[u8] = include_bytes!("./assets/doorbell-qr.png");
const DOORBELL_QR_POINTER: &[u8] = include_bytes!("./assets/qr-pointer.svg");

pub fn float32_lerp(source: f32, destination: f32, percent: f32) -> f32 {
    return source * (1.0 - percent) + destination * percent;
}
pub fn colour_lerp(source: Color, destination: Color, percent: f32) -> Color {
    return Color {
        r: float32_lerp(source.r, destination.r, percent),
        g: float32_lerp(source.g, destination.g, percent),
        b: float32_lerp(source.b, destination.b, percent),
        a: float32_lerp(source.a, destination.a, percent),
    };
}

pub fn gui_entry(nfc_messages: Receiver<i32>) {
    macroquad::Window::from_config(
        Conf {
            window_title: "Door Opener".to_owned(),
            fullscreen: false,
            window_width: 720,
            window_height: 720,
            window_resizable: false,
            sample_count: 0,
            ..Default::default()
        },
        gui_main(nfc_messages),
    )
}

async fn gui_main(nfc_messages: Receiver<i32>) {
    // let mut led_controller = LEDController::new();

    let mut queued_auth_state: (i32, bool) = (-1, false);
    let mut animating_auth_state: TimedVariable<(i32, bool)> = TimedVariable::new((-1, false));
    let mut auth_state: TimedVariable<i32> = TimedVariable::new(0);
    let mut show_welcome: TimedVariable<bool> = TimedVariable::new(true);
    let mut active_message: TimedVariable<i32> = TimedVariable::new(0);

    let mut welcome_opacity: f32 = 255.0;
    let mut accepted_opacity: f32 = 0.0;
    let mut rejected_opacity: f32 = 0.0;

    let segoe_ui = load_ttf_font_from_bytes(SEGOE_UI_FONT).unwrap();

    let doorbell_qr: Texture2D = Texture2D::from_file_with_format(DOORBELL_QR, None);
    let doorbell_qr_pointer = svg::svg_to_texture(String::from_utf8(DOORBELL_QR_POINTER.to_vec()).unwrap().as_str());

    let background_data = background::initialise_background().await;

    let mut passport_data = passport::initialise_passport().await;

    loop {
        let check_time = get_time();

        animating_auth_state.check_for_updates(check_time);
        auth_state.check_for_updates(check_time);
        show_welcome.check_for_updates(check_time);
        active_message.check_for_updates(check_time);

        if animating_auth_state.get().1 {
            animating_auth_state.set((animating_auth_state.get().0, false), -1.0);

            if animating_auth_state.get().0 > -1 {
                // led_controller.set_colour(animating_auth_state.get().0);

                match animating_auth_state.get().0 {
                    0 => {
                        auth_state.set(0, -1.0);
                        show_welcome.set(true, -1.0);

                        animating_auth_state.set((-1, true), 1.0);
                    }
                    1 => {
                        show_welcome.set(false, -1.0);
                        active_message.set(0, -1.0);

                        auth_state.set(1, 0.5);
                        animating_auth_state.set((-1, true), 1.5); // after previous + 1.0s
                    }
                    2 => {
                        auth_state.set(2, -1.0);
                        active_message.set(1, -1.0);

                        show_welcome.set(true, 1.5);
                        active_message.set(0, 6.5); // after welcome + 5.0s
                        animating_auth_state.set((-1, true), 2.0); // after welcome + 0.5s
                    }
                    3 => {
                        auth_state.set(3, -1.0);
                        active_message.set(2, -1.0);

                        show_welcome.set(true, 1.5);
                        active_message.set(0, 11.5); // after welcome + 10.0s
                        animating_auth_state.set((-1, true), 2.0); // after previous + 0.5s
                    }
                    _default => {}
                }
            }
        }

        if animating_auth_state.get().1 || queued_auth_state.1 {
            // i think i fixed it // if animations progress too fast, this is probably the problem lmao
            animating_auth_state.set((animating_auth_state.get().0, false), -1.0);
            queued_auth_state.1 = false;

            if (animating_auth_state.get().0 < 0) && (queued_auth_state.0 >= 0) {
                animating_auth_state.set((queued_auth_state.0, true), -1.0);

                queued_auth_state = (-1, false);
            }
        }

        match nfc_messages.try_recv() {
            Ok(x) => {
                queued_auth_state = (x, true);
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => (),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // probably display the error message somehow
            }
        };

        let delta_time: f32 = get_frame_time();

        clear_background(Color::from_hex(0x0a0a0a));

        background::draw_background(&background_data);

        welcome_opacity = f32::clamp(welcome_opacity + (255.0 * 2.0 * (if show_welcome.get() && (active_message.get() == 0) { 1.0 } else { -1.0 })) * delta_time, 0.0, 255.0);
        accepted_opacity = f32::clamp(accepted_opacity + (255.0 * 2.0 * (if show_welcome.get() && (active_message.get() == 1) { 1.0 } else { -1.0 })) * delta_time, 0.0, 255.0);
        rejected_opacity = f32::clamp(rejected_opacity + (255.0 * 2.0 * (if show_welcome.get() && (active_message.get() == 2) { 1.0 } else { -1.0 })) * delta_time, 0.0, 255.0);

        draw_welcome_window(welcome_opacity as u8, &segoe_ui, &doorbell_qr, &doorbell_qr_pointer);
        draw_accepted_window(accepted_opacity as u8, &segoe_ui);
        draw_rejected_window(rejected_opacity as u8, &segoe_ui, &doorbell_qr);

        draw_passport(
            360.0,
            match auth_state.get() {
                0 => 1200.0,
                1 => 360.0,
                2 => -1200.0,
                3 => 1200.0,
                _default => 1200.0,
            },
            auth_state.get(),
            &mut passport_data,
        );

        if is_key_down(KeyCode::Escape) {
            return;
        }

        next_frame().await
    }
}

fn draw_welcome_window(opacity: u8, font: &Font, doorbell_qr: &Texture2D, doorbell_qr_pointer: &Texture2D) {
    draw_rectangle(0.0, 164.0, 720.0, 392.0, Color::from_rgba(10, 10, 10, opacity));
    draw_rectangle(0.0, 164.0, 720.0, 4.0, Color::from_rgba(251, 203, 59, opacity));
    draw_rectangle(0.0, 552.0, 720.0, 4.0, Color::from_rgba(251, 203, 59, opacity));

    let _ = draw_text(
        "Welcome to Hack Night",
        32.0,
        203.0,
        648.0,
        Color::from_rgba(251, 203, 59, opacity),
        &font,
        96,
        1.0
    );
    let _ = draw_text(
        "Scan your passport to start",
        32.0,
        422.0,
        500.0,
        Color::from_rgba(251, 203, 59, opacity),
        &font,
        48,
        1.0
    );

    draw_texture_ex(
        doorbell_qr,
        580.0,
        422.0,
        Color::from_rgba(255, 255, 255, opacity),
        DrawTextureParams {
            dest_size: Some(Vec2 {
                x: 96.0,
                y: 96.0
            }),
            source: Option::None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: Option::None,
        },
    );
    draw_texture_ex(
        doorbell_qr_pointer,
        540.0,
        358.0,
        Color::from_rgba(255, 255, 255, opacity),
        DrawTextureParams {
            dest_size: Some(Vec2 {
                x: 160.0,
                y: 64.0
            }),
            source: Option::None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: Option::None,
        },
    );
}

fn draw_accepted_window(opacity: u8, font: &Font) {
    draw_rectangle(0.0, 212.0, 720.0, 296.0, Color::from_rgba(10, 10, 10, opacity));
    draw_rectangle(0.0, 212.0, 720.0, 4.0, Color::from_rgba(251, 203, 59, opacity));
    draw_rectangle(0.0, 504.0, 720.0, 4.0, Color::from_rgba(251, 203, 59, opacity));

    let _ = draw_text(
        "Welcome back!",
        32.0,
        251.0,
        648.0,
        Color::from_rgba(251, 203, 59, opacity),
        &font,
        96,
        1.0
    );
    let _ = draw_text(
        "Please be mindful of the door opening",
        32.0,
        374.0,
        648.0,
        Color::from_rgba(251, 203, 59, opacity),
        &font,
        48,
        1.0
    );
}

fn draw_rejected_window(opacity: u8, font: &Font, doorbell_qr: &Texture2D) {
    draw_rectangle(0.0, 140.0, 720.0, 440.0, Color::from_rgba(10, 10, 10, opacity));
    draw_rectangle(0.0, 140.0, 720.0, 4.0, Color::from_rgba(251, 203, 59, opacity));
    draw_rectangle(0.0, 576.0, 720.0, 4.0, Color::from_rgba(251, 203, 59, opacity));

    let _ = draw_text(
        "Invalid Passport!",
        32.0,
        179.0,
        420.0,
        Color::from_rgba(251, 203, 59, opacity),
        &font,
        96,
        1.0
    );
    let _ = draw_text(
        "Please try again or scan the QR code to ring the doorbell manually!",
        32.0,
        398.0,
        648.0,
        Color::from_rgba(251, 203, 59, opacity),
        &font,
        48,
        1.0
    );

    draw_texture_ex(
        doorbell_qr,
        500.0,
        179.0,
        Color::from_rgba(255, 255, 255, opacity),
        DrawTextureParams {
            dest_size: Some(Vec2 {
                x: 192.0,
                y: 192.0
            }),
            source: Option::None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: Option::None,
        },
    );
}