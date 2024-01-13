use serialport::SerialPort;

pub const SERVO_ID_ALL: u8 = 0xfe;

const SERVO_MOVE_TIME_WRITE: u8 = 1;
const SERVO_MOVE_TIME_READ: u8 = 2;
const SERVO_MOVE_TIME_WAIT_WRITE: u8 = 7;
const SERVO_MOVE_TIME_WAIT_READ: u8 = 8;
const SERVO_MOVE_START: u8 = 11;
const SERVO_MOVE_STOP: u8 = 12;
const SERVO_ID_WRITE: u8 = 13;
const SERVO_ID_READ: u8 = 14;
const SERVO_ANGLE_OFFSET_ADJUST: u8 = 17;
const SERVO_ANGLE_OFFSET_WRITE: u8 = 18;
const SERVO_ANGLE_OFFSET_READ: u8 = 19;
const SERVO_ANGLE_LIMIT_WRITE: u8 = 20;
const SERVO_ANGLE_LIMIT_READ: u8 = 21;
const SERVO_VIN_LIMIT_WRITE: u8 = 22;
const SERVO_VIN_LIMIT_READ: u8 = 23;
const SERVO_TEMP_MAX_LIMIT_WRITE: u8 = 24;
const SERVO_TEMP_MAX_LIMIT_READ: u8 = 25;
const SERVO_TEMP_READ: u8 = 26;
const SERVO_VIN_READ: u8 = 27;
const SERVO_POS_READ: u8 = 28;
const SERVO_OR_MOTOR_MODE_WRITE: u8 = 29;
const SERVO_OR_MOTOR_MODE_READ: u8 = 30;
const SERVO_LOAD_OR_UNLOAD_WRITE: u8 = 31;
const SERVO_LOAD_OR_UNLOAD_READ: u8 = 32;
const SERVO_LED_CTRL_WRITE: u8 = 33;
const SERVO_LED_CTRL_READ: u8 = 34;
const SERVO_LED_ERROR_WRITE: u8 = 35;
const SERVO_LED_ERROR_READ: u8 = 36;

const SERVO_ERROR_OVER_TEMPERATURE: u8 = 1;
const SERVO_ERROR_OVER_VOLTAGE: u8 = 2;
const SERVO_ERROR_LOCKED_ROTOR: u8 = 4;

fn lower_byte(value: u16) -> u8 {
    return (value % 256) as u8;
}

fn higher_byte(value: u16) -> u8 {
    return ((value / 256) % 256) as u8;
}

fn word(lower: u8, higher: u8) -> u16 {
    return (lower as u16) + ((higher as u16) * 256);
}
pub struct ServoController {
    serial_port: Box<dyn SerialPort>,
}

impl ServoController {
    pub fn new(port_name: String) -> ServoController {
        let serial_port = serialport::new(port_name, 115_200)
            .timeout(std::time::Duration::from_millis(10))
            .open()
            .expect("Failed to open port");

        std::thread::sleep(std::time::Duration::from_millis(3000));

        return ServoController {
            serial_port: serial_port,
        };
    }

    fn _command(&mut self, servo_id: u8, command: u8, parameters: Vec<u8>) {
        let length: u8 = 3 + parameters.len() as u8;

        let mut parameter_sum: i32 = 0;

        for parameter in parameters.clone() {
            parameter_sum += parameter as i32;
        }

        let checksum: u8 = (255
            - ((servo_id as i32 + length as i32 + command as i32 + parameter_sum) % 256))
            as u8;

        self.serial_port
            .write([0x55, 0x55, servo_id, length as u8, command].as_ref());
        self.serial_port.write(parameters.as_slice());
        self.serial_port.write([checksum].as_ref());
    }

    // I don't feel like implementing these rn, maybe later
    //     def _wait_for_response(&mut self, servo_id, command, timeout=None):
    //         timeout = Timeout(timeout or self._timeout)

    //         def read(size=1):
    //             self._serial.timeout = timeout.time_left()
    //             data = self._serial.read(size)
    //             if len(data) != size:
    //                 raise TimeoutError()
    //             return data

    //         while True:
    //             data = []
    //             data += read(1)
    //             if data[-1] != 0x55:
    //                 continue
    //             data += read(1)
    //             if data[-1] != 0x55:
    //                 continue
    //             data += read(3)
    //             sid = data[2]
    //             length = data[3]
    //             cmd = data[4]
    //             if length > 7:
    //                 LOGGER.error('Invalid length for packet %s', list(data))
    //                 continue

    //             data += read(length-3) if length > 3 else []
    //             params = data[5:]
    //             data += read(1)
    //             checksum = data[-1]
    //             if 255-(sid + length + cmd + sum(params)) % 256 != checksum:
    //                 LOGGER.error('Invalid checksum for packet %s', list(data))
    //                 continue

    //             if cmd != command:
    //                 LOGGER.warning('Got unexpected command %s response %s',
    //                                cmd, list(data))
    //                 continue

    //             if servo_id != SERVO_ID_ALL and sid != servo_id:
    //                 LOGGER.warning('Got command response from unexpected servo %s', sid)
    //                 continue

    //             return [sid, cmd, *params]

    //     def _query(&mut self, servo_id, command, timeout=None):
    //         with self._lock:
    //             self._command(servo_id, command)
    //             return self._wait_for_response(servo_id, command, timeout=timeout)

    //     def get_servo_id(&mut self, servo_id=SERVO_ID_ALL, timeout=None):
    //         response = self._query(servo_id, SERVO_ID_READ, timeout=timeout)
    //         return response[2]

    pub fn set_servo_id(&mut self, servo_id: u8, new_servo_id: u8) {
        self._command(servo_id, SERVO_ID_WRITE, vec![new_servo_id]);
    }

    pub fn move_now(&mut self, servo_id: u8, position: u16, time: u16) {
        let position = position.clamp(0, 1000);
        let time = time.clamp(0, 30000);

        self._command(
            servo_id,
            SERVO_MOVE_TIME_WRITE,
            vec![
                lower_byte(position),
                higher_byte(position),
                lower_byte(time),
                higher_byte(time),
            ],
        )
    }

    //     def get_prepared_move(&mut self, servo_id, timeout=None):
    //         """Returns servo position and time tuple"""
    //         response = self._query(servo_id, SERVO_MOVE_TIME_WAIT_READ, timeout=timeout)
    //         return word(response[2], response[3]), word(response[4], response[5])

    pub fn move_prepare(&mut self, servo_id: u8, position: u16, time: u16) {
        let position = position.clamp(0, 1000);
        let time = time.clamp(0, 30000);

        self._command(
            servo_id,
            SERVO_MOVE_TIME_WAIT_WRITE,
            vec![
                lower_byte(position),
                higher_byte(position),
                lower_byte(time),
                higher_byte(time),
            ],
        )
    }

    pub fn move_start(&mut self, servo_id: u8) {
        self._command(servo_id, SERVO_MOVE_START, vec![]);
    }

    pub fn move_stop(&mut self, servo_id: u8) {
        self._command(servo_id, SERVO_MOVE_STOP, vec![]);
    }

    //     def get_position_offset(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_ANGLE_OFFSET_READ, timeout=timeout)
    //         deviation = response[2]
    //         if deviation > 127:
    //             deviation -= 256
    //         return deviation

    pub fn set_position_offset(&mut self, servo_id: u8, deviation: u8) {
        self._command(servo_id, SERVO_ANGLE_OFFSET_ADJUST, vec![deviation]);
    }

    pub fn save_position_offset(&mut self, servo_id: u8) {
        self._command(servo_id, SERVO_ANGLE_OFFSET_WRITE, vec![]);
    }

    //     def get_position_limits(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_ANGLE_LIMIT_READ, timeout=timeout)
    //         return word(response[2], response[3]), word(response[4], response[5])

    pub fn set_position_limits(&mut self, servo_id: u8, min_position: i16, max_position: i16) {
        let min_position = min_position.clamp(-1000, 1000) as i32;
        let max_position = max_position.clamp(-1000, 1000) as i32;

        let min_position_ranged = if min_position >= 0 {
            min_position as u16
        } else {
            (min_position + 65536) as u16
        };
        let max_position_ranged = if max_position >= 0 {
            max_position as u16
        } else {
            (max_position + 65536) as u16
        };

        self._command(
            servo_id,
            SERVO_ANGLE_LIMIT_WRITE,
            vec![
                lower_byte(min_position_ranged),
                higher_byte(min_position_ranged),
                lower_byte(max_position_ranged),
                higher_byte(max_position_ranged),
            ],
        )
    }

    //     def get_voltage_limits(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_VIN_LIMIT_READ, timeout=timeout)
    //         return word(response[2], response[3]), word(response[4], response[5])

    pub fn set_voltage_limits(&mut self, servo_id: u8, min_voltage: u16, max_voltage: u16) {
        let min_voltage = min_voltage.clamp(4500, 12000);
        let max_voltage = max_voltage.clamp(4500, 12000);

        self._command(
            servo_id,
            SERVO_VIN_LIMIT_WRITE,
            vec![
                lower_byte(min_voltage),
                higher_byte(min_voltage),
                lower_byte(max_voltage),
                higher_byte(max_voltage),
            ],
        )
    }

    //     def get_max_temperature_limit(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_TEMP_MAX_LIMIT_READ, timeout=timeout)
    //         return response[2]

    pub fn set_max_temperature_limit(&mut self, servo_id: u8, max_temperature: u8) {
        let max_temperature = max_temperature.clamp(50, 100);

        self._command(servo_id, SERVO_TEMP_MAX_LIMIT_WRITE, vec![max_temperature]);
    }

    //     def get_temperature(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_TEMP_READ, timeout=timeout)
    //         return response[2]

    //     def get_voltage(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_VIN_READ, timeout=timeout)
    //         return word(response[2], response[3])

    //     def get_position(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_POS_READ, timeout=timeout)
    //         position = word(response[2], response[3])
    //         if position > 32767:
    //             position -= 65536
    //         return position

    //     def get_mode(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_OR_MOTOR_MODE_READ, timeout=timeout)
    //         return response[2]

    //     def get_motor_speed(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_OR_MOTOR_MODE_READ, timeout=timeout)
    //         if response[2] != 1:
    //             return 0
    //         speed = word(response[4], response[5])
    //         if speed > 32767:
    //             speed -= 65536
    //         return speed

    pub fn set_servo_mode(&mut self, servo_id: u8) {
        self._command(servo_id, SERVO_OR_MOTOR_MODE_WRITE, vec![0, 0, 0, 0]);
    }

    pub fn set_motor_mode(&mut self, servo_id: u8, speed: i16) {
        let speed = speed.clamp(-1000, 1000) as i32;

        let speed_ranged = if speed >= 0 {
            speed as u16
        } else {
            (speed + 65536) as u16
        };

        self._command(
            servo_id,
            SERVO_OR_MOTOR_MODE_WRITE,
            vec![lower_byte(speed_ranged), higher_byte(speed_ranged)],
        );
    }

    //     def is_motor_on(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_LOAD_OR_UNLOAD_READ, timeout=timeout)
    //         return response[2] == 1

    pub fn motor_on(&mut self, servo_id: u8) {
        self._command(servo_id, SERVO_LOAD_OR_UNLOAD_WRITE, vec![1]);
    }

    pub fn motor_off(&mut self, servo_id: u8) {
        self._command(servo_id, SERVO_LOAD_OR_UNLOAD_WRITE, vec![0]);
    }

    //     def is_led_on(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_LED_CTRL_READ, timeout=timeout)
    //         return response[2] == 0

    pub fn led_on(&mut self, servo_id: u8) {
        self._command(servo_id, SERVO_LED_CTRL_WRITE, vec![0]);
    }

    pub fn led_off(&mut self, servo_id: u8) {
        self._command(servo_id, SERVO_LED_CTRL_WRITE, vec![1]);
    }

    //     def get_led_errors(&mut self, servo_id, timeout=None):
    //         response = self._query(servo_id, SERVO_LED_ERROR_READ, timeout=timeout)
    //         return response[2]

    pub fn set_led_errors(&mut self, servo_id: u8, error: u8) {
        let error = error.clamp(0, 7);

        self._command(servo_id, SERVO_LED_ERROR_WRITE, vec![error]);
    }
}
