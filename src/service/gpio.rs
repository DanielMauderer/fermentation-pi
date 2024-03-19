use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
};

use rppal::gpio::{Gpio, IoPin, Mode};
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
    static ref PINS: HashMap<PinType, Arc<Mutex<IoPin>>> = {
        let mut m = HashMap::new();
        m.insert(
            PinType::HeatingPin,
            init_pin_mutex(PinType::HeatingPin).unwrap(),
        );
        m.insert(
            PinType::HumidifierPin,
            init_pin_mutex(PinType::HumidifierPin).unwrap(),
        );
        m.insert(PinType::Led3Pin, init_pin_mutex(PinType::Led3Pin).unwrap());
        m.insert(PinType::Led1Pin, init_pin_mutex(PinType::Led1Pin).unwrap());
        m.insert(PinType::Led2Pin, init_pin_mutex(PinType::Led2Pin).unwrap());
        m.insert(
            PinType::SensorPin,
            init_pin_mutex(PinType::SensorPin).unwrap(),
        );
        m.insert(
            PinType::Sensor2Pin,
            init_pin_mutex(PinType::Sensor2Pin).unwrap(),
        );
        m
    };
}

pub fn read_sensor_data() -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let mut array: [u8; 5] = [0; 5];
    let mut pin_lock = get_pin_save(PinType::SensorPin)?;
    pin_lock.set_mode(rppal::gpio::Mode::Output);
    match read_sensor_from_pin(&mut pin_lock, &mut array) {
        Ok(_) => {}
        Err(e) => {
            error!("Error: {}", e);
            return Err(e);
        }
    }

    return Ok((
        convert_data_to_float(((array[0] as u16) << 8) | array[1] as u16),
        convert_data_to_float(((array[2] as u16) << 8) | array[3] as u16),
    ));
}

fn get_pin_save(
    pin_number: PinType,
) -> Result<std::sync::MutexGuard<'static, IoPin>, Box<dyn std::error::Error>> {
    let pin_lock = match PINS.get(&pin_number) {
        Some(pin) => match pin.lock() {
            Ok(pin) => pin,
            Err(e) => {
                error!("Error: {}", e);
                return Err(Box::from("Pin"));
            }
        },
        None => return Err(Box::from("Pin not found")),
    };
    Ok(pin_lock)
}

fn read_sensor_from_pin(
    pin: &mut IoPin,
    array: &mut [u8; 5],
) -> Result<(), Box<dyn std::error::Error>> {
    start_signal(pin)?;
    pin.set_mode(Mode::Input);
    ready_sensor(pin)?;
    read_data(array, pin)?;
    Ok(())
}

pub fn turn_on_heating() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin_lock = get_pin_save(PinType::HeatingPin)?;

    pin_lock.set_mode(rppal::gpio::Mode::Output);
    pin_lock.set_high();
    Ok(())
}

pub fn turn_off_heating() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin_lock = get_pin_save(PinType::HeatingPin)?;

    pin_lock.set_mode(Mode::Output);
    pin_lock.set_low();
    Ok(())
}

pub fn turn_on_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin_lock = get_pin_save(PinType::HumidifierPin)?;

    pin_lock.set_mode(Mode::Output);
    pin_lock.set_high();
    Ok(())
}

pub fn turn_off_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin_lock = get_pin_save(PinType::HumidifierPin)?;

    pin_lock.set_mode(Mode::Output);
    pin_lock.set_low();
    Ok(())
}

pub fn turn_on_led(led_index: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut pin_lock = match led_index {
        1 => get_pin_save(PinType::Led1Pin)?,
        2 => get_pin_save(PinType::Led2Pin)?,
        3 => get_pin_save(PinType::Led3Pin)?,
        _ => return Err(Box::from("Index")),
    };
    pin_lock.set_mode(Mode::Output);
    pin_lock.set_high();
    Ok(())
}

pub fn turn_off_led(led_index: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mut pin_lock = match led_index {
        1 => get_pin_save(PinType::Led1Pin)?,
        2 => get_pin_save(PinType::Led2Pin)?,
        3 => get_pin_save(PinType::Led3Pin)?,
        _ => return Err(Box::from("Index")),
    };
    pin_lock.set_mode(Mode::Output);
    pin_lock.set_low();

    Ok(())
}

fn start_signal(pin: &mut IoPin) -> Result<(), Box<dyn std::error::Error>> {
    pin.set_low();
    thread::sleep(std::time::Duration::from_millis(18));
    pin.set_high();
    Ok(())
}

fn ready_sensor(pin: &IoPin) -> Result<(), Box<dyn std::error::Error>> {
    wait_for_high(pin)?;
    wait_for_low(pin)?;
    wait_for_high(pin)?;
    wait_for_low(pin)?;
    Ok(())
}

fn wait_for_low(pin: &IoPin) -> Result<(), Box<dyn std::error::Error>> {
    let timeout_start = std::time::Instant::now();
    while pin.is_high() {
        if timeout_start.elapsed().as_millis() > TIMEOUT_DURATION {
            return Err(Box::from("Timeout"));
        }
    }
    Ok(())
}

fn wait_for_high(pin: &IoPin) -> Result<(), Box<dyn std::error::Error>> {
    let timeout_start = std::time::Instant::now();
    while pin.is_low() {
        if timeout_start.elapsed().as_millis() > TIMEOUT_DURATION {
            return Err(Box::from("Timeout"));
        }
    }
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

fn init_pin_mutex(pin: PinType) -> Result<Arc<Mutex<IoPin>>, Box<dyn std::error::Error>> {
    match Gpio::new() {
        Ok(gpio) => match gpio.get(pin as u8) {
            Ok(pin) => return Ok(Arc::new(Mutex::new(pin.into_io(Mode::Output)))),
            Err(e) => {
                error!("Error: {}", e);

                return Err(Box::from(e));
            }
        },
        Err(e) => {
            error!("Error: {}", e);
            return Err(Box::from(e));
        }
    };
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
