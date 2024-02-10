use std::io::Error;

use pn532::serialport::SysTimer;
use pn532::spi::SPIInterface;
use pn532::IntoDuration;
use pn532::{requests::SAMMode, serialport::SerialPortInterface, Pn532, Request};

use crate::config::NFC_SERIAL;

pub struct NFCReader {
    pn532: Pn532<SPIInterface, SysTimer, 512>,
}

impl NFCReader {
    pub fn new() -> NFCReader {
        // let port = serialport::new(NFC_SERIAL, 500000)
        //     .timeout(std::time::Duration::from_millis(10))
        //     .open()
        //     .expect("Failed to open port");

        // let interface = SerialPortInterface { port };

        let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 500_000, Mode::Mode0)?;

        let interface: SPIInterface<_, _> = SPIInterface {
            spi,
            cs,
        };

        let timer = SysTimer::new();

        let mut pn532: Pn532<_, _, 512> = Pn532::new(interface, timer);

        if let Err(e) = pn532.process(
            &Request::sam_configuration(SAMMode::Normal, false),
            0,
            50.ms(),
        ) {
            println!("Could not initialize PN532: {e:?}")
        }

        return NFCReader { pn532 };
    }

    pub fn poll(&mut self) -> Result<bool, Error> {
        match self
            .pn532
            .process(&Request::INLIST_ONE_ISO_A_TARGET, 7, 1000.ms())
        {
            Ok(_) => {
                return Ok(true);
            }
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Poll Empty Result",
                ));
            }
        }
    }

    pub fn read(&mut self) -> Result<(i32, std::string::String), Error> {
        match self.pn532.process(&Request::ntag_read(2), 17, 50.ms()) {
            Ok(data) => {
                println!("page 2: {:?}", &data[1..]);

                match std::str::from_utf8(&data[1..]) {
                    Ok(id_string) => match id_string.parse::<i32>() {
                        Ok(id_number) => {
                            match self.pn532.process(&Request::ntag_read(3), 17, 50.ms()) {
                                Ok(data) => {
                                    println!("page 3: {:?}", &data[1..5]);

                                    match std::str::from_utf8(&data[1..]) {
                                        Ok(secret_string) => {
                                            return Ok((id_number, String::from(secret_string)));
                                        }
                                        Err(err) => {
                                            return Err(std::io::Error::new(
                                                std::io::ErrorKind::Other,
                                                err.to_string(),
                                            ));
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(std::io::Error::new(
                                        std::io::ErrorKind::Other,
                                        "Data Empty Read",
                                    ));
                                }
                            }
                        }
                        Err(err) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                err.to_string(),
                            ));
                        }
                    },
                    Err(err) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            err.to_string(),
                        ));
                    }
                }
            }
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Data Empty Read",
                ));
            }
        }
    }
}
