use rs_ws281x::{StripType, ChannelBuilder, ControllerBuilder};
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::time::Instant;

const GAMMA_CORRECTION_TABLE: [u8; 256] = [
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  1,  1,  1,
    1,  1,  1,  1,  1,  1,  1,  1,  1,  2,  2,  2,  2,  2,  2,  2,
    2,  3,  3,  3,  3,  3,  3,  3,  4,  4,  4,  4,  4,  5,  5,  5,
    5,  6,  6,  6,  6,  7,  7,  7,  7,  8,  8,  8,  9,  9,  9, 10,
   10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16,
   17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25,
   25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36,
   37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 50,
   51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68,
   69, 70, 72, 73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89,
   90, 92, 93, 95, 96, 98, 99,101,102,104,105,107,109,110,112,114,
  115,117,119,120,122,124,126,127,129,131,133,135,137,138,140,142,
  144,146,148,150,152,154,156,158,160,162,164,167,169,171,173,175,
  177,180,182,184,186,189,191,193,196,198,200,203,205,208,210,213,
  215,218,220,223,225,228,231,233,236,239,241,244,247,249,252,255
];

// for anyone that's confused later, these are in BGR- order
const WAIT_COLOUR: [u8; 4] = [ 59, 203, 251, 0 ];
const ACCEPTED_COLOUR: [u8; 4] = [ 94, 197, 34, 0 ];
const REJECTED_COLOUR: [u8; 4] = [ 68, 68, 239, 0 ];


pub fn u8_lerp(source: u8, destination: u8, percent: f32) -> u8 {
    return ((source as f32) * (1.0 - percent) + (destination as f32) * percent) as u8;
}

pub fn raw_colour_lerp(source: [u8; 4], destination: [u8; 4], percent: f32) -> [u8; 4] {
    return [
        u8_lerp(source[0], destination[0], percent),
        u8_lerp(source[1], destination[1], percent),
        u8_lerp(source[2], destination[2], percent),
        u8_lerp(source[3], destination[3], percent),
    ];
}

pub struct LEDController {
    tx: Sender<i32>,
}

impl LEDController {
    pub fn new() -> LEDController {
        let (tx, rx) = channel::<i32>();

        let mut current_colour_state = 0;
        let mut current_colour = WAIT_COLOUR;
        let mut loop_start_time = Instant::now();

        thread::spawn(move || {
            let mut controller = ControllerBuilder::new()
                .freq(800_000)
                .dma(10)
                .channel(
                    0, // Channel Index
                    ChannelBuilder::new()
                        .pin(18)
                        .count(16) 
                        .strip_type(StripType::Ws2812)
                        .brightness(255)
                        .build(),
                )
                .build()
                .unwrap();

            loop {
                let delta_time = loop_start_time.elapsed();
                loop_start_time = Instant::now();

                match rx.try_recv() {
                    Ok(x) => {
                        current_colour_state = x;
                    }
                    Err(_) => {}
                };

                current_colour = raw_colour_lerp(
                    current_colour,
                    match current_colour_state {
                        1 => Self::gamma_correct(WAIT_COLOUR),
                        2 => Self::gamma_correct(ACCEPTED_COLOUR),
                        3 => Self::gamma_correct(REJECTED_COLOUR),
                        _ => Self::gamma_correct(WAIT_COLOUR)
                    },
                    delta_time.as_secs_f32() * 10.0,
                );
    
                let leds = controller.leds_mut(0);

                for led in &mut *leds {
                    *led = current_colour;
                }
            
                controller.render().unwrap();
            }
        });

        return Self { tx };
    }
    
    pub fn set_colour(&mut self, colour: i32) {
        let _ = self.tx.send(colour);
    }

    pub fn gamma_correct(colour: [u8; 4]) -> [u8; 4] {
        return [
            GAMMA_CORRECTION_TABLE[colour[0] as usize],
            GAMMA_CORRECTION_TABLE[colour[1] as usize],
            GAMMA_CORRECTION_TABLE[colour[2] as usize],
            GAMMA_CORRECTION_TABLE[colour[3] as usize]
        ];
    }
}