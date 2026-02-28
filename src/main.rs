pub mod auth;
pub mod config;
pub mod enums;
pub mod gui;
pub mod hardware;
pub mod timedvariable;
#[cfg(not(debug_assertions))]
mod updater;

use std::{env, time::Duration};

use async_tungstenite::{
    tokio::connect_async,
    tungstenite::{Bytes, Message},
};
use auth::auth_entry;
use futures::prelude::*;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task,
    time::sleep,
};

use crate::{enums::AuthState, gui::gui_entry, hardware::door::DoorOpener};

#[cfg(not(debug_assertions))]
use updater::update_check;

#[dotenvy::load(path = ".env", required = true, override_ = false)]
#[tokio::main]
async fn main() {
    #[cfg(not(debug_assertions))]
    if update_check().await {
        println!("Finished updating to a newer version, closing!");
        // Quit, systemd will pick us back up
        return;
    }

    let (auth_tx, gui_rx) = unbounded_channel::<AuthState>();
    let (opener_tx, opener_rx) = unbounded_channel::<()>();
    let auth_opener = opener_tx.clone();
    let gui_opener = opener_tx.clone();

    task::spawn_blocking(move || {
        auth_entry(&auth_tx, &auth_opener);
    });

    task::spawn(ws_entry(opener_tx));

    task::spawn(opener_entry(opener_rx));

    gui_entry(gui_rx, gui_opener);
}

async fn opener_entry(mut opener_rx: UnboundedReceiver<()>) {
    let door_opener = DoorOpener::new();
    loop {
        if opener_rx.recv().await.is_some() {
            println!("opener_entry: received open message");
            door_opener.open();
        }
    }
}

async fn ws_entry(opener_tx: UnboundedSender<()>) {
    #[derive(Debug, serde::Deserialize)]
    #[serde(tag = "type")]
    enum WebSocketMessage {
        Open,
    }

    let (socket, _resp) =
        match connect_async("wss://api.purduehackers.com/phonebell/door-opener").await {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to connect to API WebSocket: {e}");
                return;
            }
        };

    let (mut write, mut read) = socket.split();

    write
        .send(Message::Text(
            env::var("DOOR_OPENER_API_KEY")
                .expect("door opener API key")
                .into(),
        ))
        .await
        .expect("write auth");

    loop {
        tokio::select! {
            () = sleep(Duration::from_secs(25)) => {
                write.send(Message::Ping(Bytes::default())).await.expect("ping");
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        if let Ok(msg) = serde_json::from_str(t.as_ref()) {
                            match msg {
                                WebSocketMessage::Open => {
                                    let _ = opener_tx.send(());
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {},
                    Some(Err(e)) => eprintln!("Received err: {e:?}"),
                    None => break,
                    _ => eprintln!("Unsupported message received! {msg:?}"),
                }
            }
        }
    }

    println!("WebSocket connection closed.");
}
