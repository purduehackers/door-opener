use std::{thread, time::Duration};

use reqwest::{Error, StatusCode};
use tokio::sync::mpsc::UnboundedSender;

use crate::{enums::AuthState, hardware::nfc::NFCReader};

use AuthState::{Idle, Invalid, NFCError, NetError, Pending, Valid};

/// Authentication thread
///
/// # Panics
///
/// Will panic if the NFC reader cannot be initialized
pub fn auth_entry(gui_sender: &UnboundedSender<AuthState>, opener_tx: &UnboundedSender<()>) {
    let mut nfc_reader: NFCReader = NFCReader::new().expect("Failed to initialize NFC reader");

    loop {
        if let Ok(target) = nfc_reader.poll() {
            let _ = gui_sender.send(Pending);

            if let Ok(data) = nfc_reader.read(target) {
                let res = check_passport_validity(data.id, &data.secret);
                thread::sleep(Duration::from_millis(2500));

                match res {
                    Ok(verified) => {
                        let _ = gui_sender.send(if verified { Valid } else { Invalid });
                        if verified {
                            println!("Passport successfully validated, sending open command...");
                            match opener_tx.send(()) {
                                Ok(()) => {}
                                Err(e) => {
                                    eprintln!("auth: failed to send open command: {e:?}");
                                }
                            }
                        }
                    }
                    Err(_) => {
                        let _ = gui_sender.send(NetError);
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(2500));
                let _ = gui_sender.send(NFCError);
            }

            thread::sleep(Duration::from_millis(5000));

            let _ = gui_sender.send(Idle);
        }

        thread::sleep(Duration::from_millis(300));
    }
}

/// Checks for passport validity
///
/// # Errors
///
/// Will error if the request to the passport server fails
///
/// # Panics
///
/// Will panic if the request does not send a valid error string
pub fn check_passport_validity(id: i32, secret: &str) -> Result<bool, Error> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .post("https://id.purduehackers.com/api/door")
        .header("Content-Type", "application/json")
        .body(format!("{{\"id\": {id}, \"secret\": \"{secret}\"}}"))
        .send();

    match res {
        Ok(res) => {
            if res.status() == StatusCode::OK {
                Ok(true)
            } else {
                println!("Got error status: {}", res.status());
                println!("Got error text: {}", res.text().unwrap());
                Ok(false)
            }
        }
        Err(e) => Err(e),
    }
}
