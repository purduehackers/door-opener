pub mod background;
pub mod colors;
pub mod font_engine;
pub mod passport;
pub mod svg;

mod constants;
mod windows;

use macroquad::prelude::*;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use self::constants::{OPACITY_MAX, OPACITY_MIN};
use self::{passport::PassportData, passport::draw_passport};

use crate::gui::windows::draw_message_windows;
use crate::{enums::AuthState, timedvariable::TimedVariable};
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

fn update_opacity(opacity: &mut f32, active: bool, delta_time: f32) {
    let direction = if active { 1.0 } else { -1.0 };
    *opacity =
        (*opacity + OPACITY_MAX * 2.0 * direction * delta_time).clamp(OPACITY_MIN, OPACITY_MAX);
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

#[allow(clippy::cast_possible_truncation)]
pub fn gui_entry(nfc_messages: UnboundedReceiver<AuthState>, opener_tx: UnboundedSender<()>) {
    macroquad::Window::from_config(
        Conf {
            window_title: "Door Opener".to_owned(),
            fullscreen: true,
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
    let mut active_message: TimedVariable<AuthState> = TimedVariable::new(AuthState::Idle);

    let mut opacities = MessageOpacities::default();

    let segoe_ui = load_ttf_font_from_bytes(SEGOE_UI_FONT).unwrap();

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
        draw_message_windows(&opacities, &segoe_ui);
        draw_passport_for_state(auth_state.get(), &mut passport_data);

        #[cfg(not(debug_assertions))]
        let _ = &opener_tx;
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
    active_message: &mut TimedVariable<AuthState>,
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
    active_message: &mut TimedVariable<AuthState>,
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
                active_message.set(AuthState::Idle, -1.0);
                auth_state.set(AuthState::Pending, 0.5);
                animating_auth_state.set(AnimationEvent::reset_trigger(), 1.5);
            }
            Valid => {
                auth_state.set(AuthState::Valid, -1.0);
                active_message.set(AuthState::Valid, -1.0);

                show_welcome.set(true, 1.5);
                active_message.set(AuthState::Idle, 6.5);
                animating_auth_state.set(AnimationEvent::reset_trigger(), 2.0);
            }
            Invalid | NetError | NFCError | DoorHWNotReady => {
                auth_state.set(anim_state, -1.0);
                active_message.set(anim_state, -1.0);

                show_welcome.set(true, 1.5);
                active_message.set(AuthState::Idle, 11.5);
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
    auth_state: AuthState,
    delta_time: f32,
) {
    update_opacity(
        &mut opacities.welcome,
        show && auth_state == AuthState::Idle,
        delta_time,
    );
    update_opacity(
        &mut opacities.accepted,
        show && auth_state == AuthState::Valid,
        delta_time,
    );
    update_opacity(
        &mut opacities.rejected,
        show && auth_state == AuthState::Invalid,
        delta_time,
    );
    update_opacity(
        &mut opacities.net_error,
        show && auth_state == AuthState::NetError,
        delta_time,
    );
    update_opacity(
        &mut opacities.nfc_error,
        show && auth_state == AuthState::NFCError,
        delta_time,
    );
    update_opacity(
        &mut opacities.doorhw_not_ready_error,
        show && auth_state == AuthState::DoorHWNotReady,
        delta_time,
    );
}

fn draw_passport_for_state(auth_state: AuthState, passport_data: &mut PassportData) {
    let target_y = match auth_state {
        AuthState::Pending => screen_height() / 2.0,
        AuthState::Valid => screen_height() * -2.0,
        _ => screen_height() * 2.0,
    };
    draw_passport(screen_width() / 2.0, target_y, auth_state, passport_data);
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
