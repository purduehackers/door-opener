mod lx16a;

use std::{
    sync::mpsc::{Sender, channel},
    thread,
};

use crate::hardware::door::lx16a::LX16A;

pub struct DoorOpener {
    tx: Sender<i32>,
}

impl Default for DoorOpener {
    fn default() -> Self {
        Self::new()
    }
}

trait OpenModule {
    fn open_door(&mut self);
}

impl DoorOpener {
    pub fn new() -> DoorOpener {
        let (tx, rx) = channel::<i32>();

        thread::spawn(move || {
            loop {
                match rx.try_recv() {
                    Ok(_x) => {
                        let mut module = LX16A::new();
                        module.open_door();
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => (),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // probably display the error message somehow
                    }
                };
            }
        });

        Self { tx }
    }

    pub fn open(&self) {
        let _ = self.tx.send(1);
    }
}
