#[cfg(feature = "ada_pusher")]
mod ada_pusher;
#[cfg(feature = "lx16a")]
mod lx16a;

use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    task,
    time,
};

#[cfg(feature = "ada_pusher")]
use crate::hardware::door::ada_pusher::AdaPusher;
#[cfg(feature = "lx16a")]
use crate::hardware::door::lx16a::LX16A;
use crate::enums::AuthState;

const OPEN_DOOR_MAX_RETRIES: u32 = 3;
const OPEN_DOOR_RETRY_DELAY: Duration = Duration::from_secs(1);

pub struct DoorOpener {
    tx: UnboundedSender<()>,
}

#[async_trait]
trait OpenModule {
    async fn open_door(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
}

async fn open_with_retry(module: &mut (dyn OpenModule + Send)) {
    for attempt in 1..=OPEN_DOOR_MAX_RETRIES {
        match module.open_door().await {
            Ok(()) => return,
            Err(e) => {
                eprintln!(
                    "open_door failed (attempt {}/{}): {e}",
                    attempt, OPEN_DOOR_MAX_RETRIES
                );
                if attempt < OPEN_DOOR_MAX_RETRIES {
                    time::sleep(OPEN_DOOR_RETRY_DELAY).await;
                }
            }
        }
    }
    eprintln!("open_door failed after {} attempts", OPEN_DOOR_MAX_RETRIES);
}

impl DoorOpener {
    pub async fn new(auth_tx: UnboundedSender<AuthState>) -> DoorOpener {
        let (tx, mut rx) = unbounded_channel::<()>();

        task::spawn(async move {
            #[cfg(not(any(feature = "ada_pusher", feature = "lx16a")))]
            panic!("No hardware feature specified. At least one must be specified");

            let mut module: Option<Box<dyn OpenModule + Send>> = None;

            #[cfg(feature = "ada_pusher")]
            let (init_tx, init_rx) = tokio::sync::oneshot::channel::<Box<dyn OpenModule + Send>>();

            #[cfg(feature = "ada_pusher")]
            task::spawn(async move {
                let pusher = AdaPusher::new().await;
                let _ = init_tx.send(Box::new(pusher));
            });

            #[cfg(all(feature = "lx16a", not(feature = "ada_pusher")))]
            {
                module = Some(Box::new(LX16A::new()));
            }

            #[cfg(feature = "ada_pusher")]
            let mut init_rx = Some(init_rx);

            loop {
                #[cfg(feature = "ada_pusher")]
                {
                    if let Some(ref mut irx) = init_rx {
                        tokio::select! {
                            result = irx => {
                                if let Ok(m) = result {
                                    module = Some(m);
                                    println!("Door module initialized successfully!");
                                }
                                init_rx = None;
                            }
                            msg = rx.recv() => {
                                match msg {
                                    Some(_) => {
                                        if let Some(ref mut m) = module {
                                            open_with_retry(m.as_mut()).await;
                                        } else {
                                            let _ = auth_tx.send(AuthState::DoorHWNotReady);
                                        }
                                    }
                                    None => return,
                                }
                            }
                        }
                        continue;
                    }
                }

                match rx.recv().await {
                    Some(_) => {
                        if let Some(ref mut m) = module {
                            open_with_retry(m.as_mut()).await;
                        } else {
                            let _ = auth_tx.send(AuthState::DoorHWNotReady);
                        }
                    }
                    None => return,
                }
            }
        });
        Self { tx }
    }

    pub fn open(&self) {
        let _ = self.tx.send(());
    }
}
