use chrono::Datelike;
use rocket::serde::json::Json;

use crate::service::database::sensor::{get_all_data, HistoricSensorData, SensorData};
use crate::service::sensor::get_sensor_data;

#[get("/historic/<start_ticks>/<end_ticks>")]
pub fn get_historic(start_ticks: u64, end_ticks: u64) -> Json<Option<Vec<HistoricSensorData>>> {
    match sensor_database(start_ticks, end_ticks) {
        Ok(db) => Json(Some(db)),
        Err(_) => return Json(None),
    }
}

#[get("/")]
pub async fn get() -> Json<Option<SensorData>> {
    match get_sensor_data() {
        Ok(sensor) => Json(Some(sensor)),
        Err(e) => {
            error!("Error: {}", e);
            Json(None)
        }
    }
}

fn sensor_database(
    start_ticks: u64,
    end_ticks: u64,
) -> Result<Vec<HistoricSensorData>, Box<dyn std::error::Error>> {
    let end_date = match chrono::NaiveDateTime::from_timestamp_opt(end_ticks as i64, 0) {
        Some(date) => date.day(),
        None => return Err(Box::from("Invalid end date")),
    };

    let mut historic_sensor_data: Vec<HistoricSensorData> = Vec::new();
    let mut current_time = start_ticks;

    let mut one_day_data = get_all_data(current_time)?;
    for data in one_day_data {
        if data.time >= start_ticks && data.time <= end_ticks {
            historic_sensor_data.push(data);
        }
    }
    current_time += 86400;

    while current_time < end_ticks {
        let current_date = match chrono::NaiveDateTime::from_timestamp_opt(current_time as i64, 0) {
            Some(date) => date.day(),
            None => return Err(Box::from("Invalid current date")),
        };
        if current_date == end_date {
            break;
        }
        one_day_data = get_all_data(current_time)?;
        historic_sensor_data.append(&mut one_day_data);
        current_time += 86400;
    }
    one_day_data = get_all_data(current_time)?;
    for data in one_day_data {
        if data.time >= start_ticks && data.time <= end_ticks {
            historic_sensor_data.push(data);
        }
    }
    Ok(historic_sensor_data)
}
