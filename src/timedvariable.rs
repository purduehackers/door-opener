#[derive(Copy, Clone, Debug)]
pub struct TimedVariable<T: Copy> {
    current_value: T,
    next_value: Option<(T, f64, f64)>,
    last_time_step: f64,
}

impl<T: Copy> TimedVariable<T> {
    pub fn new(initial_value: T) -> TimedVariable<T> {
        TimedVariable {
            current_value: initial_value,
            next_value: Option::None,
            last_time_step: 0.0,
        }
    }

    pub fn check_for_updates(&mut self, current_time: f64) {
        self.last_time_step = current_time;

        if let Some(current_check) = &self.next_value {
            if (current_time - current_check.1) >= current_check.2 {
                self.current_value = current_check.0;
                self.next_value = Option::None;
            }
        }
    }

    pub fn get(self) -> T {
        self.current_value
    }

    pub fn set(&mut self, value: T, delay: f64) {
        if delay <= 0.0 {
            self.current_value = value;
        } else {
            self.next_value = Option::Some((value, self.last_time_step, delay));
        }
    }

    pub fn cancel(mut self) {
        self.next_value = Option::None;
    }
}
