use lazy_static::lazy_static;
use std::{collections::HashMap, hash::Hash, sync::Mutex};

use rppal::gpio::{Gpio, IoPin, Mode, OutputPin, Pin};
use std::thread;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
enum PinType {
    HeatingPin = 4,
    HumidifierPin = 17,
    Led3Pin = 27,
    Led1Pin = 22,
    Led2Pin = 10,
    SensorPin = 2,
    Sensor2Pin = 3,
}

const TIMEOUT_DURATION: u128 = 300;

lazy_static! {
    static ref PIN_IN_USE: Mutex<HashMap<PinType, bool>> = {
        let mut m = HashMap::new();
        m.insert(PinType::HeatingPin, false);
        m.insert(PinType::HumidifierPin, false);
        m.insert(PinType::Led3Pin, false);
        m.insert(PinType::Led1Pin, false);
        m.insert(PinType::Led2Pin, false);
        m.insert(PinType::SensorPin, false);
        m.insert(PinType::Sensor2Pin, false);
        Mutex::new(m)
    };
}

pub fn read_sensor_data() -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let mut array: [u8; 5] = [0; 5];
    let mut pin: IoPin = get_pin(PinType::SensorPin)?.into_io(rppal::gpio::Mode::Output);
    start_signal(&mut pin)?;
    warn!("1");

    pin.set_mode(Mode::Input);
    warn!("2");

    ready_sensor(&pin)?;
    warn!("3");
    read_data(&mut array, &pin)?;
    warn!("4");

    release_pin(PinType::SensorPin)?;

    return Ok((
        convert_data_to_float(((array[0] as u16) << 8) | array[1] as u16),
        convert_data_to_float(((array[2] as u16) << 8) | array[3] as u16),
    ));
}

pub fn turn_on_heating() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(PinType::HeatingPin)?;
    pin.set_high();
    release_pin(PinType::HeatingPin)?;
    Ok(())
}

pub fn turn_off_heating() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(PinType::HeatingPin)?;
    pin.set_low();
    release_pin(PinType::HeatingPin)?;
    Ok(())
}

pub fn turn_on_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(PinType::HumidifierPin)?;
    pin.set_high();
    release_pin(PinType::HumidifierPin)?;
    Ok(())
}

pub fn turn_off_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(PinType::HumidifierPin)?;
    pin.set_low();
    release_pin(PinType::HumidifierPin)?;
    Ok(())
}

pub fn turn_on_led(led_index: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut pin: OutputPin;
    match led_index {
        1 => pin = get_pin_as_output(PinType::Led1Pin)?,
        2 => pin = get_pin_as_output(PinType::Led2Pin)?,
        3 => pin = get_pin_as_output(PinType::Led3Pin)?,
        _ => return Err(Box::from("Index")),
    };
    pin.set_high();

    match led_index {
        1 => release_pin(PinType::Led1Pin)?,
        2 => release_pin(PinType::Led2Pin)?,
        3 => release_pin(PinType::Led3Pin)?,
        _ => return Err(Box::from("Index")),
    };
    Ok(())
}

pub fn turn_off_led(led_index: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut pin: OutputPin;
    match led_index {
        1 => pin = get_pin_as_output(PinType::Led1Pin)?,
        2 => pin = get_pin_as_output(PinType::Led2Pin)?,
        3 => pin = get_pin_as_output(PinType::Led3Pin)?,
        _ => return Err(Box::from("Index")),
    };
    pin.set_low();

    match led_index {
        1 => release_pin(PinType::Led1Pin)?,
        2 => release_pin(PinType::Led2Pin)?,
        3 => release_pin(PinType::Led3Pin)?,
        _ => return Err(Box::from("Index")),
    };
    Ok(())
}

fn start_signal(pin: &mut IoPin) -> Result<(), Box<dyn std::error::Error>> {
    pin.set_low();
    thread::sleep(std::time::Duration::from_millis(18));
    pin.set_high();
    Ok(())
}

fn ready_sensor(pin: &IoPin) -> Result<(), Box<dyn std::error::Error>> {
    let timeout_start = std::time::Instant::now();

    while pin.is_high() {}
    while pin.is_low() {}
    while pin.is_high() {
        if timeout_start.elapsed().as_millis() > TIMEOUT_DURATION {
            return Err(Box::from("Timeout"));
        }
    }
    while pin.is_low() {}
    Ok(())
}

fn read_byte(pin: &IoPin) -> Result<u8, Box<dyn std::error::Error>> {
    let mut value = 0;
    let mut high_time;
    let mut timeout;
    for i in 0..8 {
        timeout = std::time::Instant::now();
        while pin.is_low() {
            if timeout.elapsed().as_millis() > TIMEOUT_DURATION {
                return Err(Box::from("Timeout"));
            }
        }
        timeout = std::time::Instant::now();
        high_time = std::time::Instant::now();
        while pin.is_high() {
            if timeout.elapsed().as_millis() > TIMEOUT_DURATION {
                return Err(Box::from("Timeout"));
            }
        }
        if high_time.elapsed().as_micros() > 30 {
            value |= 1 << (7 - i);
        }
    }
    Ok(value)
}

fn get_pin(pin: PinType) -> Result<Pin, Box<dyn std::error::Error>> {
    warn!("locking pin: {} ...", pin as u8);

    let mut pins = PIN_IN_USE.lock().unwrap();
    warn!("getting lock pin: {}", pin as u8);
    let pin_lock = pins.get_mut(&pin).unwrap();

    warn!("pin status: {}", pin_lock);

    while *pin_lock {}

    *pin_lock = true;
    warn!("locked pin: {}", pin as u8);

    match Gpio::new() {
        Ok(gpio) => match gpio.get(pin as u8) {
            Ok(pin) => return Ok(pin),
            Err(e) => {
                return Err(Box::from(e));
            }
        },
        Err(e) => {
            return Err(Box::from(e));
        }
    };
}

fn get_pin_as_output(pin_type: PinType) -> Result<OutputPin, Box<dyn std::error::Error>> {
    let mut pin = get_pin(pin_type)?.into_output();
    pin.set_reset_on_drop(false);
    release_pin(PinType::SensorPin)?;
    Ok(pin)
}

fn convert_data_to_float(data: u16) -> f32 {
    let mut result = data as f32;
    result /= 10.0;
    result
}

fn read_data(array: &mut [u8; 5], pin: &IoPin) -> Result<(), Box<dyn std::error::Error>> {
    for index in 0..array.len() {
        array[index] = read_byte(pin)?;
    }
    if array[4]
        != (array[0]
            .wrapping_add(array[1])
            .wrapping_add(array[2])
            .wrapping_add(array[3])
            & 0xFF)
    {
        return Err(Box::from("Checksum"));
    }
    Ok(())
}

fn release_pin(pin: PinType) -> Result<(), Box<dyn std::error::Error>> {
    warn!("unlocking pin: {}", pin as u8);

    let mut pins = PIN_IN_USE.lock().unwrap();
    *pins.get_mut(&pin).unwrap() = false;
    Ok(())
}
