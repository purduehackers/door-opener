pub mod auth;
pub mod config;
pub mod gui;
pub mod hardware;
pub mod timedvariable;

use std::{sync::mpsc::channel, thread};

use auth::auth_entry;

//use crate::gui::gui_entry;

use crate::hardware::led::LEDController;

fn main() {
    let (auth_tx, gui_rx) = channel::<i32>();

    thread::spawn(|| {
        auth_entry(auth_tx);
    });

    // gui_entry(gui_rx);

    // all this temporary code needs to be moved to gui_entry

    let mut led_controller = LEDController::new();

    loop {

        match gui_rx.try_recv() {
            Ok(x) => {
                println!("nfc sent us {}", x);
                
                led_controller.set_colour(x);
            }
            Err(_) => {},
        };
    }
}
