mod ada_pusher;

use std::error::Error;

use async_trait::async_trait;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    task,
};

use crate::hardware::door::ada_pusher::AdaPusher;

pub struct DoorOpener {
    tx: UnboundedSender<()>,
}

#[async_trait]
trait OpenModule {
    async fn open_door(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

impl DoorOpener {
    pub async fn new() -> DoorOpener {
        let (tx, mut rx) = unbounded_channel::<()>();

        task::spawn(async move {
            let mut module: Box<dyn OpenModule + Send> = Box::new(AdaPusher::new().await);

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
