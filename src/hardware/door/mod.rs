mod ada_pusher;
mod lx16a;

use std::error::Error;

use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    task,
};

use crate::hardware::door::ada_pusher::AdaPusher;

pub struct DoorOpener {
    tx: UnboundedSender<()>,
}

trait OpenModule {
    async fn open_door(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

impl DoorOpener {
    pub async fn new() -> DoorOpener {
        // this seems wrong, if buffer capacity is 1 then we should prolly use oneshot?
        // TODO: change
        let (tx, mut rx) = unbounded_channel::<()>();
        println!("new door opener");

        task::spawn(async move {
            println!("got to inner task spawn");
            let mut module = AdaPusher::new()
                .await
                .expect("Failed to initialize ada-pusher");

            loop {
                match rx.recv().await {
                    Some(_) => {
                        println!("received req");
                        match module.open_door().await {
                            Ok(_) => (),
                            Err(_) => {
                                // TODO: display error somehow later, figure it out
                            }
                        }
                    }
                    None => {
                        // probably display the error message somehow
                    }
                };
            }
        });

        println!("returning tx");

        Self { tx }
    }

    pub fn open(&self) {
        let _ = self.tx.send(());
    }
}
