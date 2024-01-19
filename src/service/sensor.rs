use rand::Rng;

use super::database::sensor::SensorData;

pub fn get_sensor_data() -> Result<SensorData, Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();

    let temp = rng.gen_range(20.0..=30.0);
    let hum = rng.gen_range(40.0..=60.0);

    return Ok(SensorData { temp, hum });
}
