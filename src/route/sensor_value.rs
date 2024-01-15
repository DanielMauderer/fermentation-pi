use chrono::Datelike;
use rocket::response::content;
use rocket::serde::json;

use crate::service::database::sensor::{get_all_data, HistoricSensorData, SensorData};
use crate::service::sensor;

#[get("/historic/<start_ticks>/<end_ticks>")]
pub fn get_historic(start_ticks: u64, end_ticks: u64) -> content::RawJson<String> {
    let db = sensor_database(start_ticks, end_ticks);

    match json::to_string(&db) {
        Ok(json) => content::RawJson(json),
        Err(error) => content::RawJson(format!("{{\"error\": \"{}\"}}", error)),
    }
}

#[get("/")]
pub fn get() -> content::RawJson<String> {
    let sensor = &SensorData {
        temp: (sensor::get_temperature()),
        hum: (sensor::get_humidity()),
    };
    sensor::get_temperature();
    match json::to_string(sensor) {
        Ok(json) => content::RawJson(json),
        Err(error) => content::RawJson(format!("{{\"error\": \"{}\"}}", error)),
    }
}

fn sensor_database(start_ticks: u64, end_ticks: u64) -> Vec<HistoricSensorData> {
    let end_date = chrono::NaiveDateTime::from_timestamp_opt(end_ticks as i64, 0);

    let mut historic_sensor_data: Vec<HistoricSensorData> = Vec::new();
    let mut current_time = start_ticks;

    let mut one_day_data = get_all_data(current_time);
    for data in one_day_data {
        if data.time >= start_ticks && data.time <= end_ticks {
            historic_sensor_data.push(data);
        }
    }
    current_time += 86400;

    while current_time < end_ticks {
        let current_date =
            chrono::NaiveDateTime::from_timestamp_opt(current_time as i64, 0).unwrap();
        if current_date.day() == end_date.unwrap().day() {
            break;
        }
        one_day_data = get_all_data(current_time);
        historic_sensor_data.append(&mut one_day_data);
        current_time += 86400;
    }
    one_day_data = get_all_data(current_time);
    for data in one_day_data {
        if data.time >= start_ticks && data.time <= end_ticks {
            historic_sensor_data.push(data);
        }
    }
    return historic_sensor_data;
}
