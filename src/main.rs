pub mod auth;
pub mod config;
pub mod enums;
pub mod gui;
pub mod hardware;
pub mod timedvariable;
#[cfg(not(debug_assertions))]
mod updater;
pub mod websocket;

use auth::auth_entry;
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task,
};

use crate::{enums::AuthState, gui::gui_entry, hardware::door::DoorOpener, websocket::ws_entry};

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
    let door_auth_tx = auth_tx.clone();

    task::spawn_blocking(move || {
        auth_entry(&auth_tx, &auth_opener);
    });

    task::spawn(ws_entry(move || {
        let _ = opener_tx.send(());
    }));

    task::spawn(opener_entry(opener_rx, door_auth_tx));

    gui_entry(gui_rx, gui_opener);
}

async fn opener_entry(mut opener_rx: UnboundedReceiver<()>, auth_tx: UnboundedSender<AuthState>) {
    let door_opener = DoorOpener::new(auth_tx);
    loop {
        if opener_rx.recv().await.is_some() {
            println!("opener_entry: received open message");
            door_opener.open();
        }
    }
}
