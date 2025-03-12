#![feature(error_generic_member_access)]

use std::thread;

use tokio::sync::oneshot;

mod gui;

fn main() {
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    let async_context = thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async_main(shutdown_receiver));
    });

    gui::gui_entry();

    let _ = shutdown_sender.send(());
    let _ = async_context.join();
}

async fn async_main(shutdown_receiver: oneshot::Receiver<()>) {
    let _ = shutdown_receiver.await;
}
