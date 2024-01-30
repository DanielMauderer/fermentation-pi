use async_std::task;
use pid::Pid;
use std::{thread, time::Duration};

use crate::service::{
    database::project::get_active_project,
    gpio::{turn_off_heating, turn_off_humidifier, turn_on_heating, turn_on_humidifier},
};

pub fn entry_loop() -> Result<(), Box<dyn std::error::Error>> {
    let project = get_active_project()?;
    let mut hum_pid: Pid<f32> = Pid::new(project.settings.hum, 100.0);
    let mut temp_pid: Pid<f32> = Pid::new(project.settings.temp, 100.0);

    loop {
        let sensor_data = crate::service::sensor::get_sensor_data()?;

        let next_control_output_temp = hum_pid.next_control_output(sensor_data.hum);
        warn!("Next control output temp: {:?}", next_control_output_temp);
        let next_control_output_hum = temp_pid.next_control_output(sensor_data.temp);
        warn!("Next control output hum: {:?}", next_control_output_hum);
        let hum_on_time = next_control_output_temp.output / 100.0;
        let temp_on_time = next_control_output_hum.output / 100.0;

        warn!("Hum: {}, Temp: {}", sensor_data.hum, sensor_data.temp);
        warn!("Hum_on: {}, Temp_on: {}", hum_on_time, temp_on_time);

        task::spawn(async move {
            if hum_on_time > 0.0 {
                turn_on_humidifier().unwrap();
                task::sleep(Duration::from_secs(hum_on_time as u64)).await;
                turn_off_humidifier().unwrap();
            }
            task::sleep(Duration::from_secs(1 - hum_on_time as u64)).await;
        });

        task::spawn(async move {
            if temp_on_time > 0.0 {
                turn_on_heating().unwrap();
                task::sleep(Duration::from_secs(temp_on_time as u64)).await;
                turn_off_heating().unwrap();
            }
            task::sleep(Duration::from_secs(1 - temp_on_time as u64)).await;
        });
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
