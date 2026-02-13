use std::time::Duration;

use nfc1::{Context, Device, Error, Target, target_info::TargetInfo};

pub mod parser;
pub mod structs;

use crate::hardware::nfc::{parser::parse_nfc_data, structs::PassportData};

pub struct NFCReader {
    device: Device,
}

impl NFCReader {
    pub fn new() -> Result<NFCReader, Error> {
        let context: &'static mut Context = Box::leak(Box::new(Context::new().unwrap()));
        let mut device: Device = context.open()?;

        device.initiator_init()?;
        device.set_property_bool(nfc1::Property::InfiniteSelect, true)?;
        device.set_property_bool(nfc1::Property::AutoIso144434, true)?;

        Ok(Self { device })
    }

    pub fn poll(&mut self) -> Result<Target, Error> {
        self.device.initiator_poll_target(
            &[nfc1::Modulation {
                modulation_type: nfc1::ModulationType::Iso14443a,
                baud_rate: nfc1::BaudRate::Baud106,
            }],
            0xff,
            Duration::from_millis(150),
        )
    }

    pub fn read(&mut self, target: Target) -> Result<PassportData, Error> {
        if let TargetInfo::Iso14443a(_target_info) = target.target_info {
            self.device
                .set_property_bool(nfc1::Property::EasyFraming, true)?;

            let mut passport_data: Vec<u8> = vec![];

            for n in (4..50).step_by(4) {
                match self.device.initiator_transceive_bytes(
                    &[0x30, n as u8],
                    16,
                    nfc1::Timeout::Default,
                ) {
                    Ok(data) => {
                        for byte in data {
                            passport_data.push(byte);
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            let message = match parse_nfc_data(passport_data) {
                Ok(x) => x,
                Err(_) => {
                    return Err(Error::OperationAborted);
                }
            };

            if message.records.len() != 3 {
                return Err(Error::OperationAborted);
            }

            let passport_id = match message.records[1].data.parse::<i32>() {
                Ok(x) => x,
                Err(_) => {
                    return Err(Error::OperationAborted);
                }
            };
            let passport_secret = message.records[2].data.clone();

            Ok(PassportData {
                id: passport_id,
                secret: passport_secret,
            })
        } else {
            Err(Error::DeviceNotSupported)
        }
    }
}
