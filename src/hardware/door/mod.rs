#[cfg(feature = "ada_pusher")]
mod ada_pusher;
#[cfg(feature = "lx16a")]
mod lx16a;

use std::error::Error;

use async_trait::async_trait;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    task,
};

#[cfg(feature = "ada_pusher")]
use crate::hardware::door::ada_pusher::AdaPusher;
#[cfg(feature = "lx16a")]
use crate::hardware::door::lx16a::LX16A;

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
            #[cfg(feature = "ada_pusher")]
            let mut module: Box<dyn OpenModule + Send> = Box::new(
                AdaPusher::new()
                    .await
                    .expect("Failed to initialize ada-pusher"),
            );
            #[cfg(all(feature = "lx16a", not(feature = "ada_pusher")))]
            let mut module: Box<dyn OpenModule + Send> = Box::new(LX16A::new());
            #[cfg(not(any(feature = "ada_pusher", feature = "lx16a")))]
            panic!("No hardware feature specified. At least one must be specified");

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
