use super::{database::sensor::SensorData, gpio::read_sensor_data};
const MAX_RETRIES: u8 = 5;

pub fn get_sensor_data() -> Result<SensorData, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        let sensor_result = read_sensor_data();
        if sensor_result.is_err() {
            retries += 1;
            if retries >= MAX_RETRIES {
                return Err(Box::from("Max retries exceeded"));
            }
            continue;
        }
        let (temperature, humidity) = sensor_result.unwrap();
        return Ok(SensorData {
            temp: temperature,
            hum: humidity,
        });
    }
}
