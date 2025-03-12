use std::{string::String, sync::mpsc::Sender, thread, time::Duration};

use reqwest::StatusCode;

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
                        let verified = check_passport_validity(data.0, data.1);

                        thread::sleep(Duration::from_millis(2500));

                        let _ = gui_sender.send(if verified { 2 } else { 3 });

                        if verified {
                            door_opener.open();
                        }
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(2500));

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

pub fn check_passport_validity(id: i32, secret: String) -> bool {
    let client = reqwest::blocking::Client::new();
    let res = client
        .get("https://id.purduehackers.com/api/door")
        .body(format!("{{\"id\": {id}, \"secret\": \"{secret}\"}}"))
        .send();

    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => {
                return true;
            }
            _ => {
                return false;
            }
        },
        Err(_) => {
            return false;
        }
    }
}
