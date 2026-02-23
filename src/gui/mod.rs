pub mod background;
pub mod colors;
pub mod font_engine;
pub mod passport;
pub mod svg;

use colors::{BLACK_BG, WHITE_CL, YELLOW_ACCENT};
use macroquad::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use self::{font_engine::draw_text, passport::PassportData, passport::draw_passport};
use crate::{enums::AuthState, gui::font_engine::Point, timedvariable::TimedVariable};

use AuthState::{DoorHWNotReady, Idle, Invalid, NFCError, NetError, Pending, Valid};

#[derive(Copy, Clone, Debug)]
struct AnimationEvent {
    state: Option<AuthState>,
    triggered: bool,
}

impl AnimationEvent {
    fn new() -> Self {
        Self {
            state: None,
            triggered: false,
        }
    }

    fn reset_trigger() -> Self {
        Self {
            state: None,
            triggered: true,
        }
    }
}

const SEGOE_UI_FONT: &[u8] = include_bytes!("./assets/SegoeUI.ttf");
const DOORBELL_QR: &[u8] = include_bytes!("./assets/doorbell-qr.png");
const DOORBELL_QR_POINTER: &[u8] = include_bytes!("./assets/qr-pointer.svg");

fn update_opacity(opacity: &mut f32, active: bool, delta_time: f32) {
    let direction = if active { 1.0 } else { -1.0 };
    *opacity = (*opacity + 255.0 * 2.0 * direction * delta_time).clamp(0.0, 255.0);
}

#[allow(clippy::cast_possible_truncation)]
fn opacity_to_u8(opacity: f32) -> u8 {
    let clamped = opacity.clamp(0.0, 255.0).round() as i32;
    u8::try_from(clamped).unwrap_or(0)
}

fn draw_texture_sized(texture: &Texture2D, x: f32, y: f32, color: Color, w: f32, h: f32) {
    draw_texture_ex(
        texture,
        x,
        y,
        color,
        DrawTextureParams {
            dest_size: Some(Vec2 { x: w, y: h }),
            ..Default::default()
        },
    );
}

#[must_use]
pub fn float32_lerp(source: f32, destination: f32, percent: f32) -> f32 {
    source * (1.0 - percent) + destination * percent
}

#[must_use]
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
    );
}

async fn gui_main(mut nfc_messages: UnboundedReceiver<AuthState>, opener_tx: UnboundedSender<()>) {
    let mut queued_auth_state = AnimationEvent::new();
    let mut animating_auth_state: TimedVariable<AnimationEvent> =
        TimedVariable::new(AnimationEvent::new());
    let mut auth_state: TimedVariable<AuthState> = TimedVariable::new(AuthState::Idle);
    let mut show_welcome: TimedVariable<bool> = TimedVariable::new(true);
    let mut active_message: TimedVariable<i32> = TimedVariable::new(0);

    let mut opacities = MessageOpacities::default();

    let segoe_ui = load_ttf_font_from_bytes(SEGOE_UI_FONT).unwrap();

    let doorbell_qr: Texture2D = Texture2D::from_file_with_format(DOORBELL_QR, None);
    let doorbell_qr_pointer = svg::svg_to_texture(
        String::from_utf8(DOORBELL_QR_POINTER.to_vec())
            .unwrap()
            .as_str(),
    );

    let background_data = background::initialise_background();

    let mut passport_data = passport::initialise_passport();

    loop {
        let check_time = get_time();
        update_timed_variables(
            check_time,
            &mut animating_auth_state,
            &mut auth_state,
            &mut show_welcome,
            &mut active_message,
        );
        process_animation_state(
            &mut animating_auth_state,
            &mut auth_state,
            &mut show_welcome,
            &mut active_message,
        );
        advance_queued_animation(&mut animating_auth_state, &mut queued_auth_state);
        receive_nfc_message(&mut nfc_messages, &mut queued_auth_state);

        let delta_time: f32 = get_frame_time();
        clear_background(Color::from_hex(0x000a_0a0a));

        background::draw_background(&background_data);
        update_message_opacities(
            &mut opacities,
            show_welcome.get(),
            active_message.get(),
            delta_time,
        );
        draw_message_windows(&opacities, &segoe_ui, &doorbell_qr, &doorbell_qr_pointer);
        draw_passport_for_state(auth_state.get(), &mut passport_data);

        #[cfg(debug_assertions)]
        handle_debug_open(&mut queued_auth_state, &opener_tx);

        if is_key_down(KeyCode::Escape) {
            return;
        }

        next_frame().await;
    }
}

struct MessageOpacities {
    welcome: f32,
    accepted: f32,
    rejected: f32,
    net_error: f32,
    nfc_error: f32,
    doorhw_not_ready_error: f32,
}

impl Default for MessageOpacities {
    fn default() -> Self {
        Self {
            welcome: 255.0,
            accepted: 0.0,
            rejected: 0.0,
            net_error: 0.0,
            nfc_error: 0.0,
            doorhw_not_ready_error: 0.0,
        }
    }
}

fn update_timed_variables(
    check_time: f64,
    animating_auth_state: &mut TimedVariable<AnimationEvent>,
    auth_state: &mut TimedVariable<AuthState>,
    show_welcome: &mut TimedVariable<bool>,
    active_message: &mut TimedVariable<i32>,
) {
    animating_auth_state.check_for_updates(check_time);
    auth_state.check_for_updates(check_time);
    show_welcome.check_for_updates(check_time);
    active_message.check_for_updates(check_time);
}

fn process_animation_state(
    animating_auth_state: &mut TimedVariable<AnimationEvent>,
    auth_state: &mut TimedVariable<AuthState>,
    show_welcome: &mut TimedVariable<bool>,
    active_message: &mut TimedVariable<i32>,
) {
    if !animating_auth_state.get().triggered {
        return;
    }

    animating_auth_state.set(
        AnimationEvent {
            triggered: false,
            ..animating_auth_state.get()
        },
        -1.0,
    );

    if let Some(anim_state) = animating_auth_state.get().state {
        match anim_state {
            Idle => {
                auth_state.set(AuthState::Idle, -1.0);
                show_welcome.set(true, -1.0);
                animating_auth_state.set(AnimationEvent::reset_trigger(), 1.0);
            }
            Pending => {
                show_welcome.set(false, -1.0);
                active_message.set(0, -1.0);

                auth_state.set(AuthState::Pending, 0.5);
                animating_auth_state.set(AnimationEvent::reset_trigger(), 1.5);
            }
            Valid => {
                auth_state.set(AuthState::Valid, -1.0);
                active_message.set(1, -1.0);

                show_welcome.set(true, 1.5);
                active_message.set(0, 6.5);
                animating_auth_state.set(AnimationEvent::reset_trigger(), 2.0);
            }
            Invalid | NetError | NFCError | DoorHWNotReady => {
                let msg_id = match anim_state {
                    Invalid => 2,
                    NetError => 3,
                    NFCError => 4,
                    DoorHWNotReady => 5,
                    _ => unreachable!(),
                };
                auth_state.set(anim_state, -1.0);
                active_message.set(msg_id, -1.0);

                show_welcome.set(true, 1.5);
                active_message.set(0, 11.5);
                animating_auth_state.set(AnimationEvent::reset_trigger(), 2.0);
            }
        }
    }
}

fn advance_queued_animation(
    animating_auth_state: &mut TimedVariable<AnimationEvent>,
    queued_auth_state: &mut AnimationEvent,
) {
    if !(animating_auth_state.get().triggered || queued_auth_state.triggered) {
        return;
    }

    // if animations progress too fast, this is probably the problem
    animating_auth_state.set(
        AnimationEvent {
            triggered: false,
            ..animating_auth_state.get()
        },
        -1.0,
    );
    queued_auth_state.triggered = false;

    if animating_auth_state.get().state.is_none() && queued_auth_state.state.is_some() {
        animating_auth_state.set(
            AnimationEvent {
                state: queued_auth_state.state,
                triggered: true,
            },
            -1.0,
        );
        *queued_auth_state = AnimationEvent::new();
    }
}

fn receive_nfc_message(
    nfc_messages: &mut UnboundedReceiver<AuthState>,
    queued_auth_state: &mut AnimationEvent,
) {
    if let Ok(x) = nfc_messages.try_recv() {
        *queued_auth_state = AnimationEvent {
            state: Some(x),
            triggered: true,
        };
    } else {
        // probably display the error message somehow
    }
}

fn update_message_opacities(
    opacities: &mut MessageOpacities,
    show: bool,
    msg: i32,
    delta_time: f32,
) {
    update_opacity(&mut opacities.welcome, show && msg == 0, delta_time);
    update_opacity(&mut opacities.accepted, show && msg == 1, delta_time);
    update_opacity(&mut opacities.rejected, show && msg == 2, delta_time);
    update_opacity(&mut opacities.net_error, show && msg == 3, delta_time);
    update_opacity(&mut opacities.nfc_error, show && msg == 4, delta_time);
    update_opacity(
        &mut opacities.doorhw_not_ready_error,
        show && msg == 5,
        delta_time,
    );
}

fn draw_message_windows(
    opacities: &MessageOpacities,
    font: &Font,
    doorbell_qr: &Texture2D,
    doorbell_qr_pointer: &Texture2D,
) {
    draw_welcome_window(
        opacity_to_u8(opacities.welcome),
        font,
        doorbell_qr,
        doorbell_qr_pointer,
    );
    draw_accepted_window(opacity_to_u8(opacities.accepted), font);
    draw_error_window(
        opacity_to_u8(opacities.rejected),
        font,
        doorbell_qr,
        "Invalid Passport!",
        "Please try again or scan the QR code to ring the doorbell manually!",
    );
    draw_error_window(
        opacity_to_u8(opacities.net_error),
        font,
        doorbell_qr,
        "Something went wrong!",
        "We're having connectivity issues at the moment. Please try again.",
    );
    draw_error_window(
        opacity_to_u8(opacities.nfc_error),
        font,
        doorbell_qr,
        "NFC read error!",
        "Please take away your passport, then hold it still during the scan!",
    );
    draw_error_window(
        opacity_to_u8(opacities.doorhw_not_ready_error),
        font,
        doorbell_qr,
        "Button pusher not ready yet!",
        "Try again after a minute or contact an organizer.",
    );
}

fn draw_passport_for_state(auth_state: AuthState, passport_data: &mut PassportData) {
    let target_y = match auth_state {
        AuthState::Pending => 360.0,
        AuthState::Valid => -1200.0,
        _ => 1200.0,
    };
    draw_passport(360.0, target_y, auth_state, passport_data);
}

#[cfg(debug_assertions)]
fn handle_debug_open(queued_auth_state: &mut AnimationEvent, opener_tx: &UnboundedSender<()>) {
    if is_key_pressed(KeyCode::Space) {
        println!("Opening door for debugging purposes...");
        *queued_auth_state = AnimationEvent {
            state: Some(Valid),
            triggered: true,
        };
        let _ = opener_tx.send(());
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

    draw_texture_sized(doorbell_qr, 580.0, 422.0, WHITE_CL(opacity), 96.0, 96.0);
    draw_texture_sized(
        doorbell_qr_pointer,
        540.0,
        358.0,
        WHITE_CL(opacity),
        160.0,
        64.0,
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

    draw_texture_sized(doorbell_qr, 500.0, 179.0, WHITE_CL(opacity), 192.0, 192.0);
}
