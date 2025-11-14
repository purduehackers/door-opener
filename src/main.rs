pub mod auth;
pub mod config;
pub mod enums;
pub mod gui;
pub mod hardware;
pub mod timedvariable;

use std::{sync::mpsc::channel, thread};

use auth::auth_entry;

use crate::{enums::AuthState, gui::gui_entry};

fn main() {
    let (auth_tx, gui_rx) = channel::<AuthState>();

    thread::spawn(|| {
        auth_entry(auth_tx);
    });

    gui_entry(gui_rx);
}
