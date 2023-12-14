use std::{thread, sync::mpsc::{channel, Sender}, time};

use crate::gui::gui_entry;

mod gui;

fn main() {
    let (nfc_tx, gui_rx) = channel::<i32>();

    thread::spawn(|| {
        gui_entry(gui_rx);
    });

    nfc_thread(nfc_tx);
}

fn nfc_thread(gui_sender: Sender::<i32>) {
    let mut current_auth_state = 0;
    let mut should_accept = true;
    let mut next_iter_resets = false;
    
    loop {
        let _ = gui_sender.send(current_auth_state);

        current_auth_state += 1;

        if next_iter_resets {
            next_iter_resets = false;
            current_auth_state = 0;
        } else if current_auth_state == 2 {
            next_iter_resets = true;
        
            if !should_accept {
                current_auth_state += 1;
            }
        
            should_accept = !should_accept;
        }

        thread::sleep(time::Duration::from_millis(5000));
    }
}