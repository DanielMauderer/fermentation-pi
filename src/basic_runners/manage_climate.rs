use async_std::task;
use pid::Pid;
use std::{thread, time::Duration};

use crate::service::{
    database::{project::get_active_project, sensor::SensorData},
    gpio::{turn_off_heating, turn_off_humidifier, turn_on_heating, turn_on_humidifier},
    sensor::get_sensor_data,
};

pub fn entry_loop() {
    error!("w2");

    let project = match get_active_project() {
        Ok(project) => project,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };
    let mut hum_pid: Pid<f32> = Pid::new(project.settings.hum, 100.0);
    hum_pid.p(10.0, 100.0).i(4.5, 100.0).d(0.25, 100.0);
    let mut temp_pid: Pid<f32> = Pid::new(project.settings.temp, 100.0);
    temp_pid.p(10.0, 100.0).i(4.5, 100.0).d(0.25, 100.0);

    let mut sensor_data: SensorData = match get_sensor_data() {
        Ok(sensor_data) => sensor_data,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };

    loop {
        let next_control_output_temp = hum_pid.next_control_output(sensor_data.hum);
        let next_control_output_hum = temp_pid.next_control_output(sensor_data.temp);
        let hum_on_time = next_control_output_temp.output / 100.0;
        let temp_on_time = next_control_output_hum.output / 100.0;
        warn!("hum_on_time: {}", hum_on_time);
        warn!("temp_on_time: {}", temp_on_time);
        task::spawn(async move {
            if hum_on_time > 0.0 {
                match turn_on_humidifier() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                };
                task::sleep(Duration::from_secs(hum_on_time as u64)).await;
                match turn_off_humidifier() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                };
            }
            task::sleep(Duration::from_secs(1 - hum_on_time as u64)).await;
        });

        sensor_data = match get_sensor_data() {
            Ok(sensor_data) => sensor_data,
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        };

        task::spawn(async move {
            if temp_on_time > 0.0 {
                match turn_on_heating() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                }
                task::sleep(Duration::from_secs(temp_on_time as u64)).await;
                match turn_off_heating() {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                };
            }
            task::sleep(Duration::from_secs(1 - temp_on_time as u64)).await;
        });
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
