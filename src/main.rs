pub mod auth;
pub mod config;
pub mod enums;
pub mod gui;
pub mod hardware;
pub mod timedvariable;

use std::{
    env,
    sync::mpsc::{Receiver, Sender, channel},
    thread,
};

use auth::auth_entry;
use tungstenite::{Message, connect};

use crate::{enums::AuthState, gui::gui_entry, hardware::door::DoorOpener};

#[dotenvy::load(path = ".env", required = true, override_ = false)]
fn main() {
    let (auth_tx, gui_rx) = channel::<AuthState>();
    let (opener_tx, opener_rx) = channel::<()>();

    let auth_opener = opener_tx.clone();
    thread::spawn(|| {
        auth_entry(auth_tx, auth_opener);
    });

    thread::spawn(|| {
        ws_entry(opener_tx);
    });

    thread::spawn(|| {
        opener_entry(opener_rx);
    });

    gui_entry(gui_rx);
}

fn opener_entry(opener_rx: Receiver<()>) {
    let door_opener = DoorOpener::new();
    while opener_rx.recv().is_ok() {
        door_opener.open();
    }
}

fn ws_entry(opener_tx: Sender<()>) {
    let (mut socket, _resp) = match connect("wss://api.purduehackers.com/phonebell/door-opener") {
        Ok(x) => x,
        Err(e) => {
            eprintln!("Failed to connect to API WebSocket: {e}");
            return;
        }
    };

    socket
        .write(tungstenite::Message::Text(
            env::var("DOOR_OPENER_API_KEY")
                .expect("door opener API key")
                .into(),
        ))
        .expect("write auth");

    #[derive(Debug, serde::Deserialize)]
    enum WebSocketMessage {
        Open,
    }

    while let Ok(msg) = socket.read() {
        match msg {
            Message::Text(t) => {
                if let Ok(msg) = serde_json::from_str(&t.to_string()) {
                    match msg {
                        WebSocketMessage::Open => {
                            let _ = opener_tx.send(());
                        }
                    }
                }
            }
            _ => eprintln!("Unsupported message received! {msg:?}"),
        }
    }

    println!("WebSocket connection closed.");
}
