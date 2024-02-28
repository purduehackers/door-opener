use std::time::Duration;

use nfc1::{Device, Target, Error, Context, target_info::TargetInfo};

pub mod parser;
use crate::hardware::nfc::parser::parse_nfc_data;

pub struct NFCReader<'a> {
    device: Device<'a>,
}

impl<'a> NFCReader<'a> {
    pub fn new() -> NFCReader<'a> {
        let context: &'static mut Context<'static>  = Box::leak(Box::new(Context::new().unwrap()));
        let mut device: Device<'a> = context.open().unwrap();
    
        let _ = device.initiator_init();
        let _ = device.set_property_bool(nfc1::Property::InfiniteSelect, true);
        let _ = device.set_property_bool(nfc1::Property::AutoIso144434, true);

        return Self { device };
    }

    pub fn poll(&mut self) -> Result<Target, Error> {
        return self.device.initiator_poll_target(&[nfc1::Modulation {
            modulation_type: nfc1::ModulationType::Iso14443a,
            baud_rate: nfc1::BaudRate::Baud106,
        }], 0xff, Duration::from_millis(150));
    }

    pub fn read(&mut self, target: Target) -> Result<(i32, std::string::String), Error> {
        if let TargetInfo::Iso14443a(_target_info) = target.target_info {
            let _ = self.device.set_property_bool(nfc1::Property::EasyFraming, true);

            let mut passport_data: Vec<u8> = vec![];

            for n in (4..50).step_by(4) {
                match self.device.initiator_transceive_bytes(&[0x30, n as u8], 16, nfc1::Timeout::Default) {
                    Ok(data) => {
                        for byte in data {
                            passport_data.push(byte);
                        }
                    }
                    Err(e) => {
                        println!("target read failed: {:?}", e);
                        return Err(e);
                    }
                }
            }
            
            println!("Raw Data: {:?}", passport_data);//String::from_utf8_lossy(&passport_data));

            let message = match parse_nfc_data(passport_data) {
                Ok(x) => x,
                Err(e) => {
                    println!("Parse Error: {:?}", e);
                    return Err(Error::OperationAborted);
                }
            };

            println!("NDEF Message: {:?}", message);

            return Ok((message.records[1].data.parse::<i32>().unwrap(), message.records[2].data.clone()));
        } else {
            println!("target selection failed");

            return Err(Error::DeviceNotSupported);
        }
    }
}
