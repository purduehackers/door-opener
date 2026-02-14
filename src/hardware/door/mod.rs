mod ada_pusher;
mod lx16a;

use std::error::Error;

use async_trait::async_trait;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    task,
};

use crate::hardware::door::{ada_pusher::AdaPusher, lx16a::LX16A};

pub struct DoorOpener {
    tx: UnboundedSender<()>,
}

#[async_trait]
trait OpenModule {
    async fn open_door(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

impl DoorOpener {
    pub async fn new() -> DoorOpener {
        // this seems wrong, if buffer capacity is 1 then we should prolly use oneshot?
        // TODO: change
        let (tx, mut rx) = unbounded_channel::<()>();

        task::spawn(async move {
            let mut module: Box<dyn OpenModule + Send> = if cfg!(feature = "ada_pusher") {
                Box::new(
                    AdaPusher::new()
                        .await
                        .expect("Failed to initialize ada-pusher"),
                )
            } else {
                Box::new(LX16A::new())
            };

            loop {
                match rx.recv().await {
                    Some(_) => {
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
        Self { tx }
    }

    pub fn open(&self) {
        let _ = self.tx.send(());
    }
}
