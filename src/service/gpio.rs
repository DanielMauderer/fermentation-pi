use rocket::form::error;
use rppal::gpio::{Gpio, InputPin, OutputPin, Pin};
use std::sync::Mutex;
use std::thread;

static HEATING_LOCK: Mutex<u8> = Mutex::new(0);
static HUMIDIFIER_LOCK: Mutex<u8> = Mutex::new(0);
static LED1_LOCK: Mutex<u8> = Mutex::new(0);
static LED2_LOCK: Mutex<u8> = Mutex::new(0);
static LED3_LOCK: Mutex<u8> = Mutex::new(0);
static SENSOR_LOCK: Mutex<u8> = Mutex::new(0);

const SENSOR_PIN: u8 = 27;
const HEATING_PIN: u8 = 2;
const HUMIDIFIER_PIN: u8 = 22;
const LED1_PIN: u8 = 17;
const LED2_PIN: u8 = 3;
const LED3_PIN: u8 = 4;
const TIMEOUT_DURATION: u128 = 300;
const ERROR_TIMEOUT: u8 = 253;

pub fn read_sensor_data() -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let _unused = SENSOR_LOCK.lock().unwrap();
    let mut array: [u8; 5] = [0; 5];
    start_signal()?;
    let in_pin = &get_pin(SENSOR_PIN)?.into_input();
    ready_sensor(in_pin)?;

    read_data(&mut array, in_pin)?;

    return Ok((
        convert_data_to_float(((array[0] as u16) << 8) | array[1] as u16),
        convert_data_to_float(((array[2] as u16) << 8) | array[3] as u16),
    ));
}

fn ready_sensor(pin: &InputPin) -> Result<(), Box<dyn std::error::Error>> {
    let timeout_start = std::time::Instant::now();
    info!("waiting for sensor ready");

    while pin.is_low() {}
    Ok(while pin.is_high() {
        info!("sensor sending ready");

        if timeout_start.elapsed().as_millis() > TIMEOUT_DURATION {
            return Err(Box::from("Timeout"));
        }
    })
}

pub fn read_sensor_data_rand() -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let rand1 = rand::random::<f32>();
    let rand2 = rand::random::<f32>();
    Ok((rand1, rand2))
}

pub fn turn_on_heating() -> Result<(), Box<dyn std::error::Error>> {
    let _unused = HEATING_LOCK.lock().unwrap();
    let mut pin = get_pin(HEATING_PIN)?.into_output();
    pin.set_high();
    Ok(())
}

pub fn turn_off_heating() -> Result<(), Box<dyn std::error::Error>> {
    let _unused = HEATING_LOCK.lock().unwrap();
    let mut pin = get_pin(HEATING_PIN)?.into_output();
    pin.set_low();
    Ok(())
}

pub fn turn_on_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let _unused = HUMIDIFIER_LOCK.lock().unwrap();
    let mut pin = get_pin(HUMIDIFIER_PIN)?.into_output();
    pin.set_high();
    Ok(())
}

pub fn turn_off_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let _unused = HUMIDIFIER_LOCK.lock().unwrap();
    let mut pin = get_pin(HUMIDIFIER_PIN)?.into_output();
    pin.set_low();
    Ok(())
}

pub fn turn_on_led(led_index: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut pin: OutputPin;
    match led_index {
        1 => {
            pin = {
                let _unused = LED1_LOCK.lock().unwrap();
                get_pin(LED1_PIN)?.into_output()
            }
        }
        2 => {
            let _unused = LED2_LOCK.lock().unwrap();
            pin = get_pin(LED2_PIN)?.into_output()
        }
        3 => {
            let _unused = LED3_LOCK.lock().unwrap();
            pin = get_pin(LED3_PIN)?.into_output()
        }
        _ => return Err(Box::from("Index")),
    };
    pin.set_high();
    Ok(())
}

pub fn turn_off_led(led_index: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut pin: OutputPin;
    match led_index {
        1 => {
            pin = {
                let _unused = LED1_LOCK.lock().unwrap();
                get_pin(LED1_PIN)?.into_output()
            }
        }
        2 => {
            let _unused = LED2_LOCK.lock().unwrap();
            pin = get_pin(LED2_PIN)?.into_output()
        }
        3 => {
            let _unused = LED3_LOCK.lock().unwrap();
            pin = get_pin(LED3_PIN)?.into_output()
        }
        _ => return Err(Box::from("Index")),
    };
    pin.set_low();
    Ok(())
}

fn start_signal() -> Result<(), Box<dyn std::error::Error>> {
    info!("sending start signal");
    let mut pin = get_pin(SENSOR_PIN)?.into_output();
    pin.set_low();
    thread::sleep(std::time::Duration::from_millis(18));
    pin.set_high();
    Ok(())
}

fn read_byte(pin: &InputPin) -> Result<u8, Box<dyn std::error::Error>> {
    let mut value = 0;
    let mut timeout_start = std::time::Instant::now();
    for i in 0..8 {
        turn_on_heating()?;
        info!("reading bit {}", i);
        while pin.is_low() {}
        while pin.is_high() {}
        timeout_start = std::time::Instant::now();
        while pin.is_low() {}
        if timeout_start.elapsed().as_nanos() > 30 {
            value |= 1 << (7 - i);
        }
        turn_off_heating()?;
    }
    info!("read value: {}", value);
    Ok(value)
}

fn get_pin(pin_number: u8) -> Result<Pin, Box<dyn std::error::Error>> {
    match Gpio::new() {
        Ok(gpio) => match gpio.get(pin_number) {
            Ok(pin) => return Ok(pin),
            Err(e) => {
                info!("Error: {}", e);
                return Err(Box::from(e));
            }
        },
        Err(e) => {
            info!("Error: {}", e);
            return Err(Box::from(e));
        }
    };
}

/* add floating point to a u16 3840 -> 38.40 */
fn convert_data_to_float(data: u16) -> f32 {
    let mut result = data as f32;
    result /= 10.0;
    result
}

fn read_data(array: &mut [u8; 5], pin: &InputPin) -> Result<(), Box<dyn std::error::Error>> {
    info!("reading sensor data");

    for index in 0..array.len() {
        info!("reading byte {}", index);
        array[index] = read_byte(pin)?;
        if array[index] == ERROR_TIMEOUT {
            return Err(Box::from("Timeout"));
        }
    }
    if array[4] != ((array[0] + array[1] + array[2] + array[3]) & 0xFF) {
        return Err(Box::from("Checksum"));
    }
    Ok(())
}
