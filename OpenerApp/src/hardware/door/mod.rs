mod lx16a;

use std::{
    sync::mpsc::{channel, Sender},
    thread, time,
};

use crate::config::{
    DOOR_SERVO_ID, DOOR_SERVO_PRESSED_POSITION, DOOR_SERVO_RELEASED_POSITION, DOOR_SERVO_SERIAL,
};

use self::lx16a::ServoController;

pub struct DoorOpener {
    tx: Sender<i32>,
}

impl DoorOpener {
    pub fn new() -> DoorOpener {
        let (tx, rx) = channel::<i32>();
        let mut servo_controller = ServoController::new(DOOR_SERVO_SERIAL.to_string());

        thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(_x) => {
                        servo_controller.move_now(DOOR_SERVO_ID, DOOR_SERVO_PRESSED_POSITION, 0);

                        thread::sleep(time::Duration::from_millis(1000));

                        servo_controller.move_now(DOOR_SERVO_ID, DOOR_SERVO_RELEASED_POSITION, 0);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => (),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // probably display the error message somehow
                    }
                };
            }
        });

        return Self { tx };
    }

    pub fn open(&self) {
        let _ = self.tx.send(1);
    }
}