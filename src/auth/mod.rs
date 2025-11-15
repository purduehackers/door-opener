use std::{string::String, sync::mpsc::Sender, thread, time::Duration};

use reqwest::{StatusCode, Error};

use crate::{
    enums::AuthState,
    hardware::{door::DoorOpener, nfc::NFCReader},
};

use AuthState::*;

pub fn auth_entry(gui_sender: Sender<AuthState>) {
    let mut nfc_reader: NFCReader = NFCReader::new().expect("Failed to initialize NFC reader");
    let door_opener: DoorOpener = DoorOpener::new();

    loop {
        if let Ok(target) = nfc_reader.poll() {
            let _ = gui_sender.send(Pending);

            match nfc_reader.read(target) {
                Ok(data) => {
                    let res = check_passport_validity(data.id, data.secret);

                    thread::sleep(Duration::from_millis(2500));

                    match res {
                        Ok(verified) => {
                            let _ = gui_sender.send(if verified { Valid } else { Invalid });
                            if verified {
                                door_opener.open();
                            }
                        }
                        Err(_) => {
                            let _ = gui_sender.send(NetError);
                        }
                    }
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(2500));

                    let _ = gui_sender.send(NFCError);
                }
            }

            thread::sleep(Duration::from_millis(5000));

            let _ = gui_sender.send(Idle);
        }

        thread::sleep(Duration::from_millis(300));
    }
}

pub fn check_passport_validity(id: i32, secret: String) -> Result<bool, Error> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("https://id.purduehackers.com/api/door")
        .header("Content-Type", "application/json")
        .body(format!("{{\"id\": {id}, \"secret\": \"{secret}\"}}"))
        .send();

    match res {
        Ok(res) => match res.status() {
            StatusCode::OK => Ok(true),
            _ => {
                println!("Got error status: {}", res.status());
                println!("Got error text: {}", res.text().unwrap());
                Ok(false)
            }
        },
        Err(e) => Err(e),
    }
}
