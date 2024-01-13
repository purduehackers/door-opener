pub mod auth;
pub mod config;
pub mod gui;
pub mod hardware;
pub mod timedvariable;

use std::{sync::mpsc::channel, thread};

use auth::auth_entry;

use crate::gui::gui_entry;

fn main() {
    let (auth_tx, gui_rx) = channel::<i32>();

    thread::spawn(|| {
        auth_entry(auth_tx);
    });

    // gui_entry(gui_rx);

    loop {
        match gui_rx.try_recv() {
            Ok(x) => {
                println!("nfc sent us {}", x);
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => (),
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // probably display the error message somehow
            }
        };
    }
}
