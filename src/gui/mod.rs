pub mod background;
pub mod colors;
pub mod font_engine;
pub mod passport;
pub mod svg;

use colors::*;
use macroquad::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use self::{font_engine::draw_text, passport::draw_passport};
use crate::{enums::AuthState, gui::font_engine::Point, timedvariable::TimedVariable};

use AuthState::*;

const SEGOE_UI_FONT: &[u8] = include_bytes!("./assets/SegoeUI.ttf");
const DOORBELL_QR: &[u8] = include_bytes!("./assets/doorbell-qr.png");
const DOORBELL_QR_POINTER: &[u8] = include_bytes!("./assets/qr-pointer.svg");

fn update_opacity(opacity: &mut f32, active: bool, delta_time: f32) {
    let direction = if active { 1.0 } else { -1.0 };
    *opacity = (*opacity + 255.0 * 2.0 * direction * delta_time).clamp(0.0, 255.0);
}

pub fn float32_lerp(source: f32, destination: f32, percent: f32) -> f32 {
    source * (1.0 - percent) + destination * percent
}
pub fn colour_lerp(source: Color, destination: Color, percent: f32) -> Color {
    Color {
        r: float32_lerp(source.r, destination.r, percent),
        g: float32_lerp(source.g, destination.g, percent),
        b: float32_lerp(source.b, destination.b, percent),
        a: float32_lerp(source.a, destination.a, percent),
    }
}

pub fn gui_entry(nfc_messages: UnboundedReceiver<AuthState>, opener_tx: UnboundedSender<()>) {
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
        gui_main(nfc_messages, opener_tx),
    )
}

async fn gui_main(mut nfc_messages: UnboundedReceiver<AuthState>, opener_tx: UnboundedSender<()>) {
    let mut queued_auth_state: (Option<AuthState>, bool) = (None, false);
    let mut animating_auth_state: TimedVariable<(Option<AuthState>, bool)> =
        TimedVariable::new((None, false));
    let mut auth_state: TimedVariable<AuthState> = TimedVariable::new(AuthState::Idle);
    let mut show_welcome: TimedVariable<bool> = TimedVariable::new(true);
    let mut active_message: TimedVariable<i32> = TimedVariable::new(0);

    let mut welcome_opacity: f32 = 255.0;
    let mut accepted_opacity: f32 = 0.0;
    let mut rejected_opacity: f32 = 0.0;
    let mut net_error_opacity: f32 = 0.0;
    let mut nfc_error_opacity: f32 = 0.0;

    let segoe_ui = load_ttf_font_from_bytes(SEGOE_UI_FONT).unwrap();

    let doorbell_qr: Texture2D = Texture2D::from_file_with_format(DOORBELL_QR, None);
    let doorbell_qr_pointer = svg::svg_to_texture(
        String::from_utf8(DOORBELL_QR_POINTER.to_vec())
            .unwrap()
            .as_str(),
    );

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

            if animating_auth_state.get().0.is_some() {
                match animating_auth_state.get().0.unwrap() {
                    // Welcome screen
                    Idle => {
                        auth_state.set(AuthState::Idle, -1.0);
                        show_welcome.set(true, -1.0);

                        animating_auth_state.set((None, true), 1.0);
                    }
                    // Loading screen
                    Pending => {
                        show_welcome.set(false, -1.0);
                        active_message.set(0, -1.0);

                        auth_state.set(AuthState::Pending, 0.5);
                        animating_auth_state.set((None, true), 1.5); // after previous + 1.0s
                    }
                    // Verified passport screen
                    Valid => {
                        auth_state.set(AuthState::Valid, -1.0);
                        active_message.set(1, -1.0);

                        show_welcome.set(true, 1.5);
                        active_message.set(0, 6.5); // after welcome + 5.0s
                        animating_auth_state.set((None, true), 2.0); // after welcome + 0.5s
                    }
                    // Invalid passport screen
                    Invalid => {
                        auth_state.set(AuthState::Invalid, -1.0);
                        active_message.set(2, -1.0);

                        show_welcome.set(true, 1.5);
                        active_message.set(0, 11.5); // after welcome + 10.0s
                        animating_auth_state.set((None, true), 2.0); // after previous + 0.5s
                    }
                    // Net error screen
                    NetError => {
                        auth_state.set(AuthState::NetError, -1.0);
                        active_message.set(3, -1.0);

                        show_welcome.set(true, 1.5);
                        active_message.set(0, 11.5); // after welcome + 10.0s
                        animating_auth_state.set((None, true), 2.0); // after previous + 0.5s
                    }
                    // NFC error screen
                    NFCError => {
                        auth_state.set(AuthState::NFCError, -1.0);
                        active_message.set(4, -1.0);

                        show_welcome.set(true, 1.5);
                        active_message.set(0, 11.5); // after welcome + 10.0s
                        animating_auth_state.set((None, true), 2.0); // after previous + 0.5s
                    }
                }
            }
        }

        if animating_auth_state.get().1 || queued_auth_state.1 {
            // i think i fixed it // if animations progress too fast, this is probably the problem lmao
            animating_auth_state.set((animating_auth_state.get().0, false), -1.0);
            queued_auth_state.1 = false;

            if (animating_auth_state.get().0.is_none()) && (queued_auth_state.0.is_some()) {
                animating_auth_state.set((queued_auth_state.0, true), -1.0);

                queued_auth_state = (None, false);
            }
        }

        match nfc_messages.try_recv() {
            Ok(x) => {
                queued_auth_state = (Some(x), true);
            }
            Err(_) => {
                // probably display the error message somehow
            }
        };

        let delta_time: f32 = get_frame_time();

        clear_background(Color::from_hex(0x0a0a0a));

        background::draw_background(&background_data);

        let show = show_welcome.get();
        let msg = active_message.get();
        update_opacity(&mut welcome_opacity, show && msg == 0, delta_time);
        update_opacity(&mut accepted_opacity, show && msg == 1, delta_time);
        update_opacity(&mut rejected_opacity, show && msg == 2, delta_time);
        update_opacity(&mut net_error_opacity, show && msg == 3, delta_time);
        update_opacity(&mut nfc_error_opacity, show && msg == 4, delta_time);

        draw_welcome_window(
            welcome_opacity as u8,
            &segoe_ui,
            &doorbell_qr,
            &doorbell_qr_pointer,
        );
        draw_accepted_window(accepted_opacity as u8, &segoe_ui);
        draw_error_window(
            rejected_opacity as u8,
            &segoe_ui,
            &doorbell_qr,
            "Invalid Passport!",
            "Please try again or scan the QR code to ring the doorbell manually!",
        );
        draw_error_window(
            net_error_opacity as u8,
            &segoe_ui,
            &doorbell_qr,
            "Something went wrong!",
            "We're having connectivity issues at the moment. Please try again.",
        );
        draw_error_window(
            nfc_error_opacity as u8,
            &segoe_ui,
            &doorbell_qr,
            "NFC read error!",
            "Please take away your passport, then hold it still during the scan!",
        );

        draw_passport(
            360.0,
            match auth_state.get() {
                AuthState::Idle => 1200.0,
                AuthState::Pending => 360.0,
                AuthState::Valid => -1200.0,
                _ => 1200.0,
            },
            auth_state.get(),
            &mut passport_data,
        );

        #[cfg(debug_assertions)]
        if is_key_pressed(KeyCode::Space) {
            println!("Opening door for debugging purposes...");
            queued_auth_state = (Some(Valid), true);
            let _ = opener_tx.send(());
        }

        if is_key_down(KeyCode::Escape) {
            return;
        }

        next_frame().await
    }
}

fn draw_welcome_window(
    opacity: u8,
    font: &Font,
    doorbell_qr: &Texture2D,
    doorbell_qr_pointer: &Texture2D,
) {
    draw_rectangle(0.0, 164.0, 720.0, 392.0, BLACK_BG(opacity));
    draw_rectangle(0.0, 164.0, 720.0, 4.0, YELLOW_ACCENT(opacity));
    draw_rectangle(0.0, 552.0, 720.0, 4.0, YELLOW_ACCENT(opacity));

    let _ = draw_text(
        "Welcome to Hack Night",
        Point::new(32.0, 203.0),
        648.0,
        YELLOW_ACCENT(opacity),
        font,
        96,
        1.0,
    );
    let _ = draw_text(
        "Scan your passport to start",
        Point::new(32.0, 422.0),
        500.0,
        YELLOW_ACCENT(opacity),
        font,
        48,
        1.0,
    );

    draw_texture_ex(
        doorbell_qr,
        580.0,
        422.0,
        WHITE_CL(opacity),
        DrawTextureParams {
            dest_size: Some(Vec2 { x: 96.0, y: 96.0 }),
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
        WHITE_CL(opacity),
        DrawTextureParams {
            dest_size: Some(Vec2 { x: 160.0, y: 64.0 }),
            source: Option::None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: Option::None,
        },
    );
}

fn draw_accepted_window(opacity: u8, font: &Font) {
    draw_rectangle(0.0, 212.0, 720.0, 296.0, BLACK_BG(opacity));
    draw_rectangle(0.0, 212.0, 720.0, 4.0, YELLOW_ACCENT(opacity));
    draw_rectangle(0.0, 504.0, 720.0, 4.0, YELLOW_ACCENT(opacity));

    let _ = draw_text(
        "Welcome back!",
        Point::new(32.0, 251.0),
        648.0,
        YELLOW_ACCENT(opacity),
        font,
        96,
        1.0,
    );
    let _ = draw_text(
        "Please be mindful of the door opening",
        Point::new(32.0, 374.0),
        648.0,
        YELLOW_ACCENT(opacity),
        font,
        48,
        1.0,
    );
}

fn draw_error_window(
    opacity: u8,
    font: &Font,
    doorbell_qr: &Texture2D,
    title: &str,
    subtitle: &str,
) {
    draw_rectangle(0.0, 140.0, 720.0, 440.0, BLACK_BG(opacity));
    draw_rectangle(0.0, 140.0, 720.0, 4.0, YELLOW_ACCENT(opacity));
    draw_rectangle(0.0, 576.0, 720.0, 4.0, YELLOW_ACCENT(opacity));

    let _ = draw_text(
        title,
        Point::new(32.0, 179.0),
        420.0,
        YELLOW_ACCENT(opacity),
        font,
        96,
        1.0,
    );
    let _ = draw_text(
        subtitle,
        Point::new(32.0, 398.0),
        648.0,
        YELLOW_ACCENT(opacity),
        font,
        48,
        1.0,
    );

    draw_texture_ex(
        doorbell_qr,
        500.0,
        179.0,
        WHITE_CL(opacity),
        DrawTextureParams {
            dest_size: Some(Vec2 { x: 192.0, y: 192.0 }),
            source: Option::None,
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            pivot: Option::None,
        },
    );
}
