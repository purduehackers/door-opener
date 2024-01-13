use std::{
    sync::mpsc::Sender,
    thread::{self},
    time::Duration,
};

use crate::{hardware::door::DoorOpener, config::NFC_SERIAL};

pub fn auth_entry(gui_sender: Sender<i32>) {
    let door_opener: DoorOpener = DoorOpener::new();

    let mut current_auth_state = 0;
    // let mut should_accept = true;
    // let mut next_iter_resets = false;

    let mut context = nfc1::Context::new().unwrap();
    let mut device = context.open().unwrap();

    let _ = device.initiator_init();
    let _ = device.set_property_bool(nfc1::Property::InfiniteSelect, true);

    loop {
        match device.initiator_select_passive_target(&nfc1::Modulation {
            modulation_type: nfc1::ModulationType::Iso14443a,
            baud_rate: nfc1::BaudRate::Baud106,
        }) {
            Ok(target) => {
                let _ = gui_sender.send(1);

                if let nfc1::target_info::TargetInfo::Iso14443a(target_info) = target.target_info {
                    //println!("Found ISO/IEC 14443-A target: {}", HexFmt(&target_info.uid[..target_info.uid_len]));
                    
                    let _ = device.set_property_bool(nfc1::Property::EasyFraming, true);

                    let test = device.initiator_transceive_bytes_timed([0x30, 0x00].as_mut_ptr(), 2).unwrap();

                    println!("mifare data: {:?}", test);

                    let verified = true;

                    let _ = gui_sender.send(if verified {2} else {3});

                    thread::sleep(Duration::from_millis(5000));

                    let _ = gui_sender.send(0);
                }
            }
            Err(_) => {}
        }

        thread::sleep(Duration::from_millis(300));

        // current_auth_state += 1;

        // if next_iter_resets {
        //     next_iter_resets = false;
        //     current_auth_state = 0;
        // } else if current_auth_state == 2 {
        //     next_iter_resets = true;

        //     if !should_accept {
        //         current_auth_state += 1;
        //     } else {
        //         door_opener.open();
        //     }

        //     should_accept = !should_accept;
        // }

        // thread::sleep(time::Duration::from_millis(5000));
    }
}
