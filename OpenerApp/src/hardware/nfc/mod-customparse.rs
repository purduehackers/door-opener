use std::time::Duration;

use nfc1::{Device, Target, Error, Context, target_info::{TargetInfo, Iso14443a}};

pub mod ndef;
use crate::hardware::nfc::ndef::NDEF;

pub enum NFCParseState {
    TLV,
    NDEFHeader,
    NDEFUrl,
    NDEFText
}

pub struct NFCParser {
    parser_state: NFCParseState,
    nfc_data: Vec<String>,
    current_byte_num: i32,
    total_content_length: i32,
}

impl NFCParser {
    pub fn new() -> NFCParser {
        return Self {
            parser_state: NFCParseState::TLV,
            nfc_data: vec![],
            current_byte_num: 0,
            total_content_length: 0
        };
    }

    pub fn ingest(&mut self, byte: u8) -> bool {
        let result = match (self.parser_state) {
            NFCParseState::TLV => {
                self.current_byte_num += 1;

                if self.current_byte_num >= 2 {
                    self.total_content_length = byte as i32;
                    
                    true
                } else {false}
            },
            NFCParseState::NDEFHeader => {
                self.current_byte_num += 1;

                if self.current_byte_num >= 0 {

                } 
            }
        };

        if result {
            self.current_byte_num = 0;
        }

        return result;
    }
}

pub struct NFCReader<'a> {
    device: Device<'a>,
}

impl<'a> NFCReader<'a> {
    pub fn new() -> NFCReader<'a> {
        let context: &'static mut Context<'static>  = Box::leak(Box::new(Context::new().unwrap()));
        let mut device: Device<'a> = context.open().unwrap();
    
        let _ = device.initiator_init();
        let _ = device.set_property_bool(nfc1::Property::InfiniteSelect, true);

        return Self { device };
    }

    pub fn poll(&mut self) -> Result<Target, Error> {
        return self.device.initiator_poll_target(&[nfc1::Modulation {
            modulation_type: nfc1::ModulationType::Iso14443a,
            baud_rate: nfc1::BaudRate::Baud106,
        }], 0xff, Duration::from_millis(150));
    }

    pub fn read(&mut self, target: Target) -> Result<(i32, std::string::String), Error> {
        if let TargetInfo::Iso14443a(target_info) = target.target_info {
            let _ = self.device.set_property_bool(nfc1::Property::EasyFraming, true);
            let mut at_protocol_end: bool = false;
            let mut current_byte = 4;

            //println!("Begin mifare: ");
            while current_byte <= 512 {
                match self.device.initiator_transceive_bytes(&[0x30, current_byte as u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], 16, nfc1::Timeout::Default) {
                    Ok(data) => {
                        for byte in data {
                            println!("{}", byte);
                            //passport_data.push(byte);
                            
                            //if byte == 0xFE {
                            //    found_terminate_byte = true;
                            //    println!("Terminate");
                            //    break;
                            //}
                        }

                        //Ok((4, "fNOABf7GiWZZ9nO26DjyDmgiSWbE5TgT".to_string()))
                    }
                    Err(e) => {
                        println!("target read failed: {:?}", e);
                        return Err(e);
                    }
                }

                if at_protocol_end {
                    break;
                } else {
                    current_byte += 4;
                }
            }
            //println!("\nEnd mifare");

            println!("Raw Data: {:?}", passport_data);//String::from_utf8_lossy(&passport_data));

            println!("NDEF Message: {:?}", message.data);

            return Ok((4, "fNOABf7GiWZZ9nO26DjyDmgiSWbE5TgT".to_string()));
        } else {
            println!("target selection failed");

            return Err(Error::DeviceNotSupported);
        }
    }
}
