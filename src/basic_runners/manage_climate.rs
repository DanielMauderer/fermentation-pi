use async_std::task;
use pid::Pid;
use std::{thread, time::Duration};

use crate::service::{
    database::{project::get_active_project, sensor::SensorData},
    gpio::{turn_off_heating, turn_off_humidifier, turn_on_heating, turn_on_humidifier},
    sensor::get_sensor_data,
};

const TEMP_DUTY_CYCLE: f32 = 1.0;
const HUM_DUTY_CYCLE: f32 = 10.0;

pub fn entry_loop_hum() {
    let project = match get_active_project() {
        Ok(project) => project,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };
    let mut hum_pid: Pid<f32> = Pid::new(project.settings.hum, 100.0);
    hum_pid.p(10.0, 100.0).i(4.5, 100.0).d(0.25, 100.0);

    let mut sensor_data: SensorData = match get_sensor_data() {
        Ok(sensor_data) => sensor_data,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    loop {
        let hum_on_time = hum_pid.next_control_output(sensor_data.hum).output / 100.0;
        warn!("hum_on_time: {}", hum_on_time);
        task::spawn(async move {
            if hum_on_time > 0.0 {
                match turn_on_humidifier() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                };
                task::sleep(Duration::from_secs_f32(HUM_DUTY_CYCLE * hum_on_time)).await;
                match turn_off_humidifier() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                };
            }
            task::sleep(Duration::from_secs_f32(
                HUM_DUTY_CYCLE - (HUM_DUTY_CYCLE * hum_on_time),
            ))
            .await;
        });

        thread::sleep(std::time::Duration::from_secs_f32(HUM_DUTY_CYCLE));
        sensor_data = match get_sensor_data() {
            Ok(sensor_data) => sensor_data,
            Err(e) => {
                error!("Error: {}", e);
                continue;
            }
        };
    }
}

pub fn entry_loop_temp() {
    let project = match get_active_project() {
        Ok(project) => project,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };
    let mut temp_pid: Pid<f32> = Pid::new(project.settings.temp, 100.0);
    info!("temp_pid: {:?}", project.settings.temp);
    temp_pid.p(2.5, 100.0).i(0.2, 100.0).d(2.5, 100.0);

    let mut sensor_data: SensorData = match get_sensor_data() {
        Ok(sensor_data) => sensor_data,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };
    match turn_off_heating() {
        Ok(_) => {}
        Err(e) => {
            error!("Error: {}", e);
        }
    };
    loop {
        let temp_on_time = temp_pid.next_control_output(sensor_data.temp).output / 100.0;
        warn!(
            "target_temp: {} current_temp: {} temp_on_time: {}",
            project.settings.temp, sensor_data.temp, temp_on_time
        );
        task::spawn(async move {
            if temp_on_time > 0.0 {
                match turn_on_heating() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                }
                task::sleep(Duration::from_secs_f32(TEMP_DUTY_CYCLE * temp_on_time)).await;
                if temp_on_time < 0.995 {
                    match turn_off_heating() {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error: {}", e);
                        }
                    };
                }
            }
            task::sleep(Duration::from_secs_f32(
                TEMP_DUTY_CYCLE - (TEMP_DUTY_CYCLE * temp_on_time),
            ))
            .await;
        });
        thread::sleep(std::time::Duration::from_secs_f32(TEMP_DUTY_CYCLE));
        sensor_data = match get_sensor_data() {
            Ok(sensor_data) => sensor_data,
            Err(e) => {
                error!("Error: {}", e);
                continue;
            }
        };
    }
}
