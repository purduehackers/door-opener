use std::collections::BTreeSet;
use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::Manager;
use btleplug::platform::{Adapter, Peripheral};
use tokio::time;
use uuid::{Uuid, uuid};

use crate::hardware::door::OpenModule;

pub struct AdaPusher {
    device: Peripheral,
    chars: BTreeSet<Characteristic>,
}

//const ADA_PUSHER_UUID: Uuid = uuid_from_u16(0xADAD);
const ADA_PUSHER_COMMAND_UUID: Uuid = uuid!("7e783540-f3ab-431f-adff-566767b8bb31");

impl AdaPusher {
    pub async fn new() -> Self {
        loop {
            match Self::try_init().await {
                Ok(pusher) => return pusher,
                Err(e) => {
                    eprintln!("ada-pusher init failed: {e}, retrying in 5s...");
                    time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn try_init() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().nth(0).unwrap();

        central.start_scan(ScanFilter::default()).await?;
        println!("Scanning for BLE devices...");
        time::sleep(Duration::from_secs(10)).await;

        let device = Self::find_ada_pusher_device(&central)
            .await
            .ok_or("ada-pusher not found during scan")?;
        println!("ada-pusher found!");

        device.connect().await?;
        println!("ada-pusher connected!");

        device.discover_services().await?;
        let chars = device.characteristics();

        Ok(AdaPusher { device, chars })
    }

    fn get_cmd_char(&self) -> Result<&Characteristic, Box<dyn Error + Send + Sync>> {
        Ok(self
            .chars
            .iter()
            .find(|c| ADA_PUSHER_COMMAND_UUID == c.uuid)
            .ok_or("failed to find command characteristic")?)
    }

    async fn find_ada_pusher_device(central: &Adapter) -> Option<Peripheral> {
        for p in central.peripherals().await.unwrap() {
            let local_names = p.properties().await.unwrap().unwrap().local_name;
            if local_names
                .iter()
                .any(|name| name.contains("ada-pusher") || name.contains("nimble"))
            {
                return Some(p);
            }
        }
        None
    }
}

#[async_trait]
impl OpenModule for AdaPusher {
    async fn open_door(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Sending open command over BLE...");
        let open_cmd = b"open".to_vec();
        self.device
            .write(self.get_cmd_char()?, &open_cmd, WriteType::WithoutResponse)
            .await?;
        println!("Command sent over BLE!");
        Ok(())
    }
}
