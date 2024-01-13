pub mod background;
pub mod passport;
pub mod svg;

use std::sync::mpsc::Receiver;

use macroquad::prelude::*;

use self::passport::draw_passport;
use crate::timedvariable::TimedVariable;

const SEGOE_UI_FONT: &[u8] = include_bytes!("./SegoeUI.ttf");

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
            sample_count: 4,
            ..Default::default()
        },
        gui_main(nfc_messages),
    )
}

async fn gui_main(nfc_messages: Receiver<i32>) {
    let mut queued_auth_state: (i32, bool) = (-1, false);
    let mut animating_auth_state: TimedVariable<(i32, bool)> = TimedVariable::new((-1, false));
    let mut auth_state: TimedVariable<i32> = TimedVariable::new(0);
    let mut show_welcome: TimedVariable<bool> = TimedVariable::new(true);
    let mut active_message: TimedVariable<i32> = TimedVariable::new(0);

    let segoe_ui = load_ttf_font_from_bytes(SEGOE_UI_FONT).unwrap();

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
                //invoke('set_led_effect', { number: animating_auth_state });

                println!("ooooooooooooooo animat {}", animating_auth_state.get().0);

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
            println!("the queue has been updated {}", queued_auth_state.0);
            animating_auth_state.set((animating_auth_state.get().0, false), -1.0);
            queued_auth_state.1 = false;

            if (animating_auth_state.get().0 < 0) && (queued_auth_state.0 >= 0) {
                animating_auth_state.set((queued_auth_state.0, true), -1.0);
                println!("we're updating to {:?}", queued_auth_state);
                println!("we really at {:?}", animating_auth_state);

                queued_auth_state = (-1, false);
            }
        }

        match nfc_messages.try_recv() {
            Ok(x) => {
                println!("we did a thing {}", x);
                queued_auth_state = (x, true);
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => (),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // probably display the error message somehow
            }
        };

        clear_background(Color::from_hex(0x0a0a0a));

        background::draw_background(&background_data);

        draw_text_window(
            "Welcome to\nHack Night",
            "Scan your passport to\nstart",
            false,
            1.0,
            &segoe_ui,
        );

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

        next_frame().await
    }
}

fn draw_text_window(title: &str, description: &str, _show_qr: bool, _opacity: f32, font: &Font) {
    draw_text_ex(
        title,
        20.0,
        200.0,
        TextParams {
            font_size: 96,
            color: Color::from_hex(0xfbcb3b),
            font: Some(&font),
            ..Default::default()
        },
    );
    draw_text_ex(
        description,
        20.0,
        300.0,
        TextParams {
            font_size: 48,
            color: Color::from_hex(0xfbcb3b),
            font: Some(&font),
            ..Default::default()
        },
    );
}
