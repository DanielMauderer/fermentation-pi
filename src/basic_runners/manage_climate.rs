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
const PID_LIMIT: f32 = 100.0;

pub fn entry_loop_hum() {
    let project = match get_active_project() {
        Ok(project) => project,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };
    let mut hum_pid: Pid<f32> = Pid::new(project.settings.hum, PID_LIMIT);
    hum_pid
        .p(35.0, PID_LIMIT)
        .i(0.09, PID_LIMIT)
        .d(10.0, PID_LIMIT);

    let mut sensor_data: SensorData = match get_sensor_data() {
        Ok(sensor_data) => sensor_data,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    loop {
        let hum_on_percentage = hum_pid.next_control_output(sensor_data.hum).output / PID_LIMIT;
        let hum_on_time = hum_on_percentage * HUM_DUTY_CYCLE;
        let hum_off_time = HUM_DUTY_CYCLE - hum_on_time;
        warn!("hum_on_time: {}", hum_on_time);
        task::spawn(async move {
            if hum_on_percentage > 0.0 {
                match turn_on_humidifier() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                };
                task::sleep(Duration::from_secs_f32(hum_on_time)).await;
                if hum_on_percentage < 0.995 {
                    match turn_off_humidifier() {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error: {}", e);
                        }
                    };
                }
            }
            task::sleep(Duration::from_secs_f32(hum_off_time)).await;
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
    let mut temp_pid: Pid<f32> = Pid::new(project.settings.temp, PID_LIMIT);
    info!("temp_pid: {:?}", project.settings.temp);
    temp_pid
        .p(35.0, PID_LIMIT)
        .i(0.09, PID_LIMIT)
        .d(10.0, PID_LIMIT);

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
        let temp_on_percentage = temp_pid.next_control_output(sensor_data.temp).output / PID_LIMIT;
        let temp_on_time = temp_on_percentage * TEMP_DUTY_CYCLE;
        let temp_off_time = TEMP_DUTY_CYCLE - temp_on_time;
        warn!(
            "target_temp: {} current_temp: {} temp_on_time: {}",
            project.settings.temp, sensor_data.temp, temp_on_time
        );
        task::spawn(async move {
            if temp_on_percentage > 0.0 {
                match turn_on_heating() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                }
                task::sleep(Duration::from_secs_f32(temp_on_time)).await;
                if temp_on_percentage < 0.995 {
                    match turn_off_heating() {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error: {}", e);
                        }
                    };
                }
            }
            task::sleep(Duration::from_secs_f32(temp_off_time)).await;
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
