use std::collections::BTreeSet;
use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use btleplug::api::{
    Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Peripheral};
use btleplug::{api::bleuuid::uuid_from_u16, platform::Manager};
use tokio::time;
use uuid::Uuid;

use crate::hardware::door::OpenModule;

pub struct AdaPusher {
    device: Peripheral,
    chars: BTreeSet<Characteristic>,
}

const ADA_PUSHER_UUID: Uuid = uuid_from_u16(0xADAD);
const ADA_PUSHER_COMMAND_UUID: Uuid = uuid_from_u16(0xADAE);

impl AdaPusher {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        println!("Initializing...");
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().nth(0).unwrap();

        central.start_scan(ScanFilter::default()).await?;
        println!("Scanning for BLE devices...");
        time::sleep(Duration::from_secs(10)).await;

        let device = Self::find_ada_pusher_device(&central)
            .await
            .ok_or("ada-pusher not found during scan")?;

        device.connect().await?;
        device.discover_services().await?;

        let chars = device.characteristics();

        Ok(AdaPusher { device, chars })
    }

    fn get_cmd_char(&self) -> Result<&Characteristic, Box<dyn Error + Send + Sync>> {
        Ok(self
            .chars
            .iter()
            .find(|c| ADA_PUSHER_COMMAND_UUID == c.uuid)
            .ok_or_else(|| "failed to find command characteristic")?)
    }

    async fn find_ada_pusher_device(central: &Adapter) -> Option<Peripheral> {
        for p in central.peripherals().await.unwrap() {
            println!("peripheral in!");
            let local_names = p.properties().await.unwrap().unwrap().local_name;
            println!("local names: {:?}", local_names);
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
        println!("someone says open sesame, sending command over BLE...");
        let open_cmd = vec![0x6F, 0x70, 0x65, 0x6E];
        self.device
            .write(self.get_cmd_char()?, &open_cmd, WriteType::WithoutResponse)
            .await?;
        println!("door opening, yay!");
        Ok(())
    }
}
