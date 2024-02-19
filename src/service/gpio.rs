use rppal::gpio::{Gpio, IoPin, Mode, OutputPin, Pin};
use std::thread;

#[repr(u8)]
enum PinType {
    HeatingPin = 2,
    HumidifierPin = 3,
    Led3Pin = 4,
    Led1Pin = 17,
    Led2Pin = 22,
    SensorPin = 27,
}
const TIMEOUT_DURATION: u128 = 300;

pub fn read_sensor_data() -> Result<(f32, f32), Box<dyn std::error::Error>> {
    let mut array: [u8; 5] = [0; 5];
    let mut pin: IoPin = get_pin(PinType::SensorPin)?.into_io(rppal::gpio::Mode::Output);
    start_signal(&mut pin)?;
    pin.set_mode(Mode::Input);
    ready_sensor(&pin)?;
    read_data(&mut array, &pin)?;

    return Ok((
        convert_data_to_float(((array[0] as u16) << 8) | array[1] as u16),
        convert_data_to_float(((array[2] as u16) << 8) | array[3] as u16),
    ));
}

pub fn turn_on_heating() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(PinType::HeatingPin)?;
    pin.set_high();
    Ok(())
}

pub fn turn_off_heating() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(PinType::HeatingPin)?;
    pin.set_low();
    Ok(())
}

pub fn turn_on_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(PinType::HumidifierPin)?;
    pin.set_high();
    Ok(())
}

pub fn turn_off_humidifier() -> Result<(), Box<dyn std::error::Error>> {
    let mut pin = get_pin_as_output(PinType::HumidifierPin)?;
    pin.set_low();
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
