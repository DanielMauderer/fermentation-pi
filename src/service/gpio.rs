use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::thread;

const SENSOR_PIN: u8 = 4;
const TIMEOUT_DURATION: u128 = 300;
const ERROR_TIMEOUT: u8 = 253;

pub fn get_sensor_data() -> Result<(f32, f32), Box<dyn std::error::Error>> {
    thread::sleep(std::time::Duration::from_millis(60));
    let mut array: [u8; 5] = [0; 5];
    start_signal()?;

    let timeout_start = std::time::Instant::now();

    let pin = get_pin_as_input()?;
    while pin.is_high() {
        if timeout_start.elapsed().as_millis() > TIMEOUT_DURATION {
            return Err(Box::from("Timeout"));
        }
    }

    read_data(&mut array)?;

    return Ok((
        convert_data_to_float(((array[0] as u16) << 8) | array[1] as u16),
        convert_data_to_float(((array[2] as u16) << 8) | array[3] as u16),
    ));
}

fn start_signal() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output()?;

    pin.set_low();
    thread::sleep(std::time::Duration::from_millis(18));
    pin.set_high();
    thread::sleep(std::time::Duration::from_micros(40));
    pin.set_low();
    Ok(())
}

fn read_byte() -> Result<u8, Box<dyn std::error::Error>> {
    let pin = get_pin_as_input()?;
    let mut value = 0;
    for i in 0..8 {
        while pin.is_low() {}
        thread::sleep(std::time::Duration::from_micros(30));
        if pin.is_high() {
            value |= 1 << (7 - i);
        }
        while pin.is_high() {}
    }
    Ok(value)
}

fn get_pin_as_input() -> Result<InputPin, Box<dyn std::error::Error>> {
    match Gpio::new() {
        Ok(gpio) => match gpio.get(SENSOR_PIN) {
            Ok(pin) => return Ok(pin.into_input()),
            Err(e) => {
                println!("Error: {}", e);
                return Err(Box::from(e));
            }
        },
        Err(e) => {
            println!("Error: {}", e);
            return Err(Box::from(e));
        }
    };
}

fn get_pin_as_output() -> Result<OutputPin, Box<dyn std::error::Error>> {
    match Gpio::new() {
        Ok(gpio) => match gpio.get(SENSOR_PIN) {
            Ok(pin) => return Ok(pin.into_output()),
            Err(e) => {
                println!("Error: {}", e);
                return Err(Box::from(e));
            }
        },
        Err(e) => {
            println!("Error: {}", e);
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
    let pin = get_pin_as_input()?;

    if pin.is_low() {
        thread::sleep(std::time::Duration::from_micros(80));
        if pin.is_high() {
            thread::sleep(std::time::Duration::from_micros(80));
            for index in 0..array.len() {
                array[index] = read_byte()?;
                if array[index] == ERROR_TIMEOUT {
                    return Err(Box::from("Timeout"));
                }
            }
            if array[4] != ((array[0] + array[1] + array[2] + array[3]) & 0xFF) {
                return Err(Box::from("Checksum"));
            }
        }
    }
    Ok(())
}
