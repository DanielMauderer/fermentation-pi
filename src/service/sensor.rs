use super::{database::sensor::SensorData, gpio::read_sensor_data};

pub fn get_sensor_data() -> Result<SensorData, Box<dyn std::error::Error>> {
    let (temperature, humidity) = read_sensor_data()?;

    return Ok(SensorData {
        temp: temperature,
        hum: humidity,
    });
}
