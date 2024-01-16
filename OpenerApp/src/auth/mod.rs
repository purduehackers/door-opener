use std::{
    sync::mpsc::Sender,
    thread::{self},
    time::Duration,
};

use crate::hardware::{door::DoorOpener, nfc::NFCReader};

pub fn auth_entry(gui_sender: Sender<i32>) {
    let mut nfc_reader: NFCReader = NFCReader::new();
    let door_opener: DoorOpener = DoorOpener::new();

    loop {
        match nfc_reader.poll() {
            Ok(target) => {
                let _ = gui_sender.send(1);

                match nfc_reader.read(target) {
                    Ok(data) => {
                        println!("mifare data: {:?}", data);

                        let verified = true;

                        thread::sleep(Duration::from_millis(1500));

                        let _ = gui_sender.send(if verified {2} else {3});
                        
                        if verified {
                            door_opener.open();
                        }
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(1500));

                        let _ = gui_sender.send(3);
                    }
                }

                thread::sleep(Duration::from_millis(5000));

                let _ = gui_sender.send(0);
            }
            Err(_) => {}
        }

        thread::sleep(Duration::from_millis(300));
    }
}