pub mod auth;
pub mod config;
pub mod gui;
pub mod hardware;
pub mod timedvariable;

use std::{sync::mpsc::{channel, Sender}, thread, time};

use auth::auth_entry;

use crate::gui::gui_entry;

fn main() {
    let (auth_tx, gui_rx) = channel::<i32>();

    thread::spawn(|| {
        auth_entry(auth_tx);
        // gui_tester_thread(auth_tx);
    });

    gui_entry(gui_rx);
}

// fn gui_tester_thread(gui_sender: Sender::<i32>) {
//     let mut current_auth_state = 0;
//     let mut should_accept = true;
//     let mut next_iter_resets = false;
    
//     loop {
//         let _ = gui_sender.send(current_auth_state);

//         current_auth_state += 1;

//         if next_iter_resets {
//             next_iter_resets = false;
//             current_auth_state = 0;
//         } else if current_auth_state == 2 {
//             next_iter_resets = true;
        
//             if !should_accept {
//                 current_auth_state += 1;
//             }
        
//             should_accept = !should_accept;
//         }

//         thread::sleep(time::Duration::from_millis(5000));
//     }
// }