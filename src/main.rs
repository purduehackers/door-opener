pub mod auth;
mod camera;
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
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{enums::AuthState, gui::gui_entry, hardware::door::DoorOpener, websocket::ws_entry};

#[cfg(not(debug_assertions))]
use updater::update_check;

#[dotenvy::load(path = ".env", required = true, override_ = false)]
fn main() {
    let _guard = sentry::init((
        "https://e47dea95664edd7200bbe8ba0a0c5458@o4510744753405952.ingest.us.sentry.io/4511157443362816",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            send_default_pii: true,
            ..Default::default()
        },
    ));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(sentry::integrations::tracing::layer())
        .init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            #[cfg(not(debug_assertions))]
            if update_check().await {
                info!("finished updating to a newer version, closing");
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
        });
}

async fn opener_entry(mut opener_rx: UnboundedReceiver<()>, auth_tx: UnboundedSender<AuthState>) {
    let door_opener = DoorOpener::new(auth_tx);
    loop {
        if opener_rx.recv().await.is_some() {
            info!("opener_entry received open message");
            door_opener.open();
        }
    }
}
