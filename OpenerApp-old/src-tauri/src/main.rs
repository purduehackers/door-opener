// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ptr::null_mut;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{JoinHandle, sleep};
use std::{mem, thread, time};

use fragile::Fragile;
use nfc::ffi::{
    nfc_baud_rate, nfc_connstring, nfc_modulation, nfc_modulation_type, nfc_property, nfc_target,
};
use nfc::misc;
use nfc::{context, device, initiator};
use tauri::{Manager, App};

#[derive(PartialEq, Eq)]
enum NfcMessageType {
    ShuttingDown,
    Data,
}

struct NfcMessage {
    message_type: NfcMessageType,
    message_data: Option<[u8; 265]>,
}

struct NfcThread {
    thread_handle: JoinHandle<()>,
    tx: Sender<NfcMessage>,
    rx: Receiver<NfcMessage>,
}

#[derive(Clone, serde::Serialize)]
struct AuthStatePayload {
    authState: i32,
}

#[tauri::command]
fn set_led_effect(number: i32) {
    println!("Set the LEDs to {}!", number)
}

fn start_nfc() -> Option<NfcThread> {
    let mut active_nfc_context = context::new();

    if active_nfc_context.is_null() {
        println!("Unable to initialize new NFC context!");

        return None;
    }

    nfc::init(&mut active_nfc_context);

    println!("libnfc version: {}", misc::version());

    #[allow(invalid_value)] // Yes rustc, I get C-style APIs are hard, please trust me
    let mut available_nfc_devices: [nfc_connstring; 10] =
        unsafe { mem::MaybeUninit::uninit().assume_init() };

    if nfc::list_devices(
        active_nfc_context,
        available_nfc_devices.as_mut_ptr(),
        available_nfc_devices.len(),
    ) <= 0
    {
        println!("No NFC devices are available!");

        nfc::exit(active_nfc_context);

        return None;
    }

    let active_nfc_device = nfc::open(active_nfc_context, available_nfc_devices.as_ptr());

    if active_nfc_device.is_null() {
        println!("Unable to initialize the first NFC device!");

        nfc::exit(active_nfc_context);

        return None;
    }

    if initiator::init(Box::new(active_nfc_device)) < 0 {
        println!("Cannot start the NFC device as initiator!");

        nfc::close(active_nfc_device);
        nfc::exit(active_nfc_context);

        return None;
    }

    let (outside_tx, rx): (Sender<NfcMessage>, Receiver<NfcMessage>) = channel();
    let (tx, outside_rx): (Sender<NfcMessage>, Receiver<NfcMessage>) = channel();

    // Do as I say
    let fragile_nfc_context = Fragile::new(active_nfc_context);
    let fragile_nfc_device = Fragile::new(active_nfc_device);

    let nfc_thread = thread::spawn(move || {
        let current_nfc_context = *fragile_nfc_context.get();
        let current_nfc_device = *fragile_nfc_device.get();

        let active_nfc_modulation = nfc_modulation {
            nmt: nfc_modulation_type::NMT_ISO14443A,
            nbr: nfc_baud_rate::NBR_106,
        };

        #[allow(invalid_value)]
        let active_nfc_target_info: *mut nfc_target =
            unsafe { mem::MaybeUninit::uninit().assume_init() };

        #[allow(invalid_value)]
        let mut nfc_data: [u8; 265] = unsafe { mem::MaybeUninit::uninit().assume_init() };

        loop {
            let next_recieve = rx.try_recv();

            if next_recieve.unwrap().message_type == NfcMessageType::ShuttingDown {
                break;
            }

            if initiator::select_passive_target(
                current_nfc_device,
                active_nfc_modulation,
                null_mut(),
                0,
                active_nfc_target_info,
            ) >= 0
            {
                if unsafe { (*(*active_nfc_target_info).nti.nai()).abtAtqa[0] } == 0x44 {
                    if device::set_property_bool(
                        current_nfc_device,
                        nfc_property::NP_EASY_FRAMING,
                        1,
                    ) >= 0
                    {
                        if initiator::transceive_bytes(
                            current_nfc_device,
                            [0x30, 0x00].as_mut_ptr(),
                            2,
                            nfc_data.as_mut_ptr(),
                            nfc_data.len(),
                            -1,
                        ) >= 0
                        {
                            // Finally, the data is valid here, we should send a message to the front end and do auth

                            let _ = tx.send(NfcMessage {
                                message_type: NfcMessageType::Data,
                                message_data: Some(nfc_data),
                            });

                            println!("We Recieved NFC Data:\n{:?}", nfc_data);
                        }
                    }
                }
            }
        }

        nfc::close(current_nfc_device);
        nfc::exit(current_nfc_context);
    });

    return Some(NfcThread {
        thread_handle: nfc_thread,
        tx: outside_tx,
        rx: outside_rx,
    });
}

fn stop_nfc(active_nfc_thread: NfcThread) {
    let _ = active_nfc_thread.tx.send(NfcMessage {
        message_type: NfcMessageType::ShuttingDown,
        message_data: None,
    });

    active_nfc_thread.thread_handle.join().unwrap();
}

fn setup_app(app: &mut App) {
    
}

fn main() {
    //uncomment when NFC hardware is present to enable it

    // let nfc_thread = start_nfc();

    // if nfc_thread.is_none()
    // {
    //     println!("NFC thread not started, exiting!");

    //     return;
    // }

    tauri::Builder::default()
        .setup(|app| {
            setup_app(app);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![set_led_effect])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    //uncomment when NFC hardware is present to enable it

    //stop_nfc(nfc_thread.unwrap());
}
