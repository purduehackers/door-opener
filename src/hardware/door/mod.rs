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
                        println!("Inner thread received message!");
                        match module.open_door().await {
                            Ok(_) => (),
                            Err(e) => {
                                eprintln!("Failed to open door, error: {e:?}")
                            }
                        }
                    }
                    None => {
                        eprintln!("Received nothing...");
                    }
                };
            }
        });
        println!("General door opener initialization complete; ready to receive messages");
        Self { tx }
    }

    pub fn open(&self) {
        println!("Received open command, sending to inner thread...");
        let _ = self.tx.send(());
    }
}
