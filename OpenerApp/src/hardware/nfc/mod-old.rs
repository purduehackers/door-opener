// use std::time::Duration;

// use nfc1::{Device, Target, Error, Context, target_info::{TargetInfo, Iso14443a}};

// pub struct NFCReader<'a> {
//     device: Device<'a>,
// }

// impl<'a> NFCReader<'a> {
//     pub fn new() -> NFCReader<'a> {
//         let context: &'static mut Context<'static>  = Box::leak(Box::new(Context::new().unwrap()));
//         let mut device: Device<'a> = context.open().unwrap();
    
//         let _ = device.initiator_init();
//         let _ = device.set_property_bool(nfc1::Property::InfiniteSelect, true);
    
//         return Self { device };
//     }

//     pub fn poll(&mut self) -> Result<Target, Error> {
//         return self.device.initiator_poll_target(&[nfc1::Modulation {
//             modulation_type: nfc1::ModulationType::Iso14443a,
//             baud_rate: nfc1::BaudRate::Baud106,
//         }], 0xff, Duration::from_millis(150));
//     }

//     pub fn read(&mut self, target: Target) -> Result<(Iso14443a, Vec<u8>), Error> {
//         if let TargetInfo::Iso14443a(target_info) = target.target_info {
//             let _ = self.device.set_property_bool(nfc1::Property::EasyFraming, true);

//             match self.device.initiator_transceive_bytes(&[0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 16, nfc1::Timeout::Default) {
//                 Ok(data) => {
//                     Ok((target_info, data))
//                 }
//                 Err(e) => Err(e)
//             }
//         } else {
//             Err(Error::DeviceNotSupported)
//         }
//     }
// }