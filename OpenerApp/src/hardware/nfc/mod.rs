use std::io::Error;
use std::fmt::Debug;
use core::task::Poll;

//use pn532::serialport::SysTimer;
use pn532::spi::{SPIInterface, PN532_SPI_DATAREAD, PN532_SPI_DATAWRITE, PN532_SPI_READY, PN532_SPI_STATREAD};
use pn532::{Interface, IntoDuration};
use pn532::{requests::SAMMode, Pn532, Request};

//use rppal::gpio::Mode;
//use rppal::spi::{Bus, SlaveSelect, Spi};

use std::io;
use std::io::{Read, Write};
use std::io::prelude::*;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};

//use embedded_hal::blocking::spi::{Transfer, Write};

use linux_embedded_hal::SysTimer;

use crate::config::NFC_SERIAL;


/// SPI Hardware Interface
#[derive(Clone, Debug)]
pub struct HardwareSPIInterface<SPI>
where
    SPI: Read,
    SPI: Write,
    //SPI: Transfer<u8>,
    //SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    //<SPI as Transfer<u8>>::Error: Debug,
{
    pub spi: SPI,
}

impl<SPI> Interface for HardwareSPIInterface<SPI>
where
    SPI: Read,
    SPI: Write,
    //SPI: Transfer<u8>,
    //SPI: Write<u8, Error = <SPI as Transfer<u8>>::Error>,
    //<SPI as Transfer<u8>>::Error: Debug,
{
    //type Error = <SPI as Transfer<u8>>::Error;
    type Error = std::io::Error;

    fn write(&mut self, frame: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(&[PN532_SPI_DATAWRITE])?;
        println!("write() 1: {:?}", PN532_SPI_DATAWRITE);
        self.spi.write(frame)?;
        println!("write() 2: {:?}", frame);

        // for byte in frame {
        //     self.spi.write(&[byte.reverse_bits()])?
        // }

        Ok(())
    }

    fn wait_ready(&mut self) -> Poll<Result<(), Self::Error>> {
        let mut buf = [0x00];

        self.spi.write(&[PN532_SPI_STATREAD])?;
        println!("wait_ready() 1: {:?}", PN532_SPI_STATREAD);
        println!("wait_ready() 2: {:?}", buf);
        //self.spi.transfer(&mut buf)?;
        self.spi.write(&buf)?;
        self.spi.read(&mut buf)?;
        println!("wait_ready() 3: {:?}", buf);

        if buf[0] == PN532_SPI_READY {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.write(&[PN532_SPI_DATAREAD])?;
        println!("read() 1: {:?}", PN532_SPI_DATAREAD);
        println!("read() 2: {:?}", buf);
        // self.spi.transfer(buf)?;
        self.spi.write(buf)?;
        self.spi.read(buf)?;
        println!("read() 3: {:?}", buf);
        
        // for byte in buf.iter_mut() {
        //    *byte = byte.reverse_bits();
        // }
        Ok(())
    }
    
}

pub struct NFCReader {
    pn532: Pn532<HardwareSPIInterface<Spidev>, SysTimer, 512>,
}

impl NFCReader {
    pub fn new() -> NFCReader {
        // let port = serialport::new(NFC_SERIAL, 500000)
        //     .timeout(std::time::Duration::from_millis(10))
        //     .open()
        //     .expect("Failed to open port");

        // let interface = SerialPortInterface { port };

        // let mut spi = match Spi::new(Bus::Spi0, SlaveSelect::Ss0, 500_000, rppal::spi::Mode::Mode0) {
        //     Ok(spi_device) => spi_device,
        //     Err(_) => panic!("fuck you")
        // };

        let mut spi = match Spidev::open("/dev/spidev0.0") {
            Ok(x) => x,
            Err(e) => {
                panic!("Unable to create SPI device: {:?}", e);
            }
        };

        let options = SpidevOptions::new()
             .bits_per_word(8)
             .max_speed_hz(20_000)
             .mode(SpiModeFlags::SPI_MODE_0)
             .build();
        if let Err(e) = spi.configure(&options) {
            panic!("Unable to configure SPI device: {:?}", e);
        }

        let interface: HardwareSPIInterface<_> = HardwareSPIInterface {
            spi
        };

        let timer = SysTimer::new();

        let mut pn532: Pn532<_, _, 512> = Pn532::new(interface, timer);
        
        // pn532.interface.send_wakeup_message().unwrap();

        // if let Err(e) = pn532.process(
        //     &Request::sam_configuration(SAMMode::Normal, false),
        //     1,
        //     500.ms(),
        // ) {
        //     panic!("Could not initialize PN532: {e:?}")
        // }

        if let Ok(fw) = pn532.process(
            &Request::GET_FIRMWARE_VERSION,
            8,
            1000.ms(),
        ) {
            println!("Firmware response: {:?}", fw);
        } else {
            panic!("Unable to communicate with device.");
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
