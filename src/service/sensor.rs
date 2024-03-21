use super::{database::sensor::SensorData, gpio::read_sensor_data};
const MAX_RETRIES: u8 = 5;
const MAX_TEMP: f32 = 50.0;
const MIN_TEMP: f32 = 0.0;
const MAX_HUM: f32 = 100.0;
const MIN_HUM: f32 = 0.0;

pub fn get_sensor_data() -> Result<SensorData, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        let sensor_result = read_sensor_data();
        if sensor_result.is_err() {
            retries += 1;
            if retries >= MAX_RETRIES {
                let error = sensor_result.err().unwrap();
                return Err(Box::from(format!("Max retries exceeded: {error}")));
            }
            continue;
        }
        let (humidity, temperature) = sensor_result.unwrap();
        let sensor_data = SensorData {
            temp: temperature,
            hum: humidity,
        };
        match sanity_check_sensor_data(&sensor_data) {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(Box::from(format!("Max retries exceeded: {e}")));
                }
                continue;
            }
        }
        return Ok(sensor_data);
    }
}

fn sanity_check_sensor_data(sensor_data: &SensorData) -> Result<(), Box<dyn std::error::Error>> {
    if sensor_data.temp > MAX_TEMP || sensor_data.temp < MIN_TEMP {
        return Err(Box::from("Sanity check failed"));
    }
    if sensor_data.hum > MAX_HUM || sensor_data.hum < MIN_HUM {
        return Err(Box::from("Sanity check failed"));
    }
    Ok(())
}
