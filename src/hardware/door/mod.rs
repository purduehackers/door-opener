mod lx16a;

use std::{
    sync::mpsc::{channel, Sender},
    thread, time,
};

use thiserror::Error;

use crate::config::{
    DOOR_SERVO_ID, DOOR_SERVO_PRESSED_POSITION, DOOR_SERVO_RELEASED_POSITION, DOOR_SERVO_SERIAL,
};

use self::lx16a::ServoController;

pub struct DoorOpener {
    tx: Sender<i32>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to initialize servo controller: {error}")]
    ServoControllerInitError { error: String },
}

impl DoorOpener {
    pub fn new() -> Result<DoorOpener, Error> {
        let (tx, rx) = channel::<i32>();
        let mut servo_controller = ServoController::new(DOOR_SERVO_SERIAL.to_string())
            .map_err(|e| Error::ServoControllerInitError { error: e })?;

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

        Ok(Self { tx })
    }

    pub fn open(&self) {
        let _ = self.tx.send(1);
    }
}
