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
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task,
};
use tungstenite::{Message, connect};

use crate::{enums::AuthState, gui::gui_entry, hardware::door::DoorOpener};

#[dotenvy::load(path = ".env", required = true, override_ = false)]
#[tokio::main(flavor = "multi_thread", worker_threads = 6)]
async fn main() {
    let (auth_tx, gui_rx) = unbounded_channel::<AuthState>();
    let (opener_tx, opener_rx) = unbounded_channel::<()>();
    let auth_opener = opener_tx.clone();
    let gui_opener = opener_tx.clone();

    task::spawn(async {
        auth_entry(auth_tx, auth_opener);
    });

    task::spawn(async {
        ws_entry(opener_tx);
    });

    task::spawn(opener_entry(opener_rx));

    gui_entry(gui_rx, gui_opener);
}

async fn opener_entry(mut opener_rx: UnboundedReceiver<()>) {
    let door_opener = DoorOpener::new().await;
    loop {
        match opener_rx.recv().await {
            Some(_) => door_opener.open(),
            None => {}
        }
    }
}

fn ws_entry(opener_tx: UnboundedSender<()>) {
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
    #[serde(tag = "type")]
    enum WebSocketMessage {
        Open,
    }

    while let Ok(msg) = socket.read() {
        match msg {
            Message::Text(t) => {
                if let Ok(msg) = serde_json::from_str(t.as_ref()) {
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
