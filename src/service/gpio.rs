use rocket::form::error;
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::sync::Mutex;
use std::thread;

static HEATING_LOCK: Mutex<u8> = Mutex::new(0);
static HUMIDIFIER_LOCK: Mutex<u8> = Mutex::new(0);
static LED1_LOCK: Mutex<u8> = Mutex::new(0);
static LED2_LOCK: Mutex<u8> = Mutex::new(0);
static LED3_LOCK: Mutex<u8> = Mutex::new(0);
static SENSOR_LOCK: Mutex<u8> = Mutex::new(0);

const SENSOR_PIN: u8 = 27;
const HEATING_PIN: u8 = 17;
const HUMIDIFIER_PIN: u8 = 22;
const LED1_PIN: u8 = 2;
const LED2_PIN: u8 = 3;
const LED3_PIN: u8 = 4;
const TIMEOUT_DURATION: u128 = 300;
const ERROR_TIMEOUT: u8 = 253;

pub fn read_sensor_data() -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let _unused = SENSOR_LOCK.lock().unwrap();
    thread::sleep(std::time::Duration::from_millis(60));
    let mut array: [u8; 5] = [0; 5];
    start_signal()?;

    ready_sensor()?;

    read_data(&mut array)?;

    return Ok((
        convert_data_to_float(((array[0] as u16) << 8) | array[1] as u16),
        convert_data_to_float(((array[2] as u16) << 8) | array[3] as u16),
    ));
}

fn ready_sensor() -> Result<(), Box<dyn std::error::Error>> {
    let timeout_start = std::time::Instant::now();
    info!("waiting for sensor ready");

    let pin = get_pin_as_input(SENSOR_PIN)?;
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
    let mut pin = get_pin_as_output(HEATING_PIN)?;
    pin.set_high();
    Ok(())
}

pub fn turn_off_heating() -> Result<(), Box<dyn std::error::Error>> {
    let _unused = HEATING_LOCK.lock().unwrap();
    let mut pin = get_pin_as_output(HEATING_PIN)?;
    pin.set_low();
    Ok(())
}

pub fn turn_on_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let _unused = HUMIDIFIER_LOCK.lock().unwrap();
    let mut pin = get_pin_as_output(HUMIDIFIER_PIN)?;
    pin.set_high();
    Ok(())
}

pub fn turn_off_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let _unused = HUMIDIFIER_LOCK.lock().unwrap();
    let mut pin = get_pin_as_output(HUMIDIFIER_PIN)?;
    pin.set_low();
    Ok(())
}

pub fn turn_on_led(led_index: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut pin: OutputPin;
    match led_index {
        1 => {
            pin = {
                let _unused = LED1_LOCK.lock().unwrap();
                get_pin_as_output(LED1_PIN)?
            }
        }
        2 => {
            let _unused = LED2_LOCK.lock().unwrap();
            pin = get_pin_as_output(LED2_PIN)?
        }
        3 => {
            let _unused = LED3_LOCK.lock().unwrap();
            pin = get_pin_as_output(LED3_PIN)?
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
                get_pin_as_output(LED1_PIN)?
            }
        }
        2 => {
            let _unused = LED2_LOCK.lock().unwrap();
            pin = get_pin_as_output(LED2_PIN)?
        }
        3 => {
            let _unused = LED3_LOCK.lock().unwrap();
            pin = get_pin_as_output(LED3_PIN)?
        }
        _ => return Err(Box::from("Index")),
    };
    pin.set_low();
    Ok(())
}

fn start_signal() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(SENSOR_PIN)?;
    info!("sending start signal");

    pin.set_low();
    thread::sleep(std::time::Duration::from_millis(18));
    pin.set_high();
    thread::sleep(std::time::Duration::from_micros(40));
    pin.set_low();
    Ok(())
}

fn read_byte() -> Result<u8, Box<dyn std::error::Error>> {
    let pin = get_pin_as_input(SENSOR_PIN)?;
    let mut value = 0;
    for i in 0..8 {
        info!("reading bit {}", i);
        while pin.is_low() {}
        thread::sleep(std::time::Duration::from_micros(30));
        if pin.is_high() {
            value |= 1 << (7 - i);
        }
    }
    info!("read value: {}", value);
    Ok(value)
}

fn get_pin_as_input(pin_number: u8) -> Result<InputPin, Box<dyn std::error::Error>> {
    match Gpio::new() {
        Ok(gpio) => match gpio.get(pin_number) {
            Ok(pin) => return Ok(pin.into_input()),
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

fn get_pin_as_output(pin_number: u8) -> Result<OutputPin, Box<dyn std::error::Error>> {
    match Gpio::new() {
        Ok(gpio) => match gpio.get(pin_number) {
            Ok(pin) => return Ok(pin.into_output()),
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

fn read_data(array: &mut [u8; 5]) -> Result<(), Box<dyn std::error::Error>> {
    info!("reading sensor data");

    for index in 0..array.len() {
        info!("reading byte {}", index);
        array[index] = read_byte()?;
        if array[index] == ERROR_TIMEOUT {
            return Err(Box::from("Timeout"));
        }
    }
    if array[4] != ((array[0] + array[1] + array[2] + array[3]) & 0xFF) {
        return Err(Box::from("Checksum"));
    }
    Ok(())
}
