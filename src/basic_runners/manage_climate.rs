use async_std::task;
use pid::Pid;
use std::time::Duration;

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

        let hum_output = hum_pid.next_control_output(sensor_data.hum).output;
        let temp_output = temp_pid.next_control_output(sensor_data.temp).output;

        print!("Hum: {}, Temp: {}", hum_output, temp_output);

        if hum_output > 0.0 {
            turn_on_humidifier()?;
        }
        if temp_output > 0.0 {
            turn_on_heating()?;
        }

        task::spawn(async move {
            let sleep_time = 1.0 / hum_output.abs();
            task::sleep(Duration::from_secs(sleep_time as u64)).await;
            turn_off_heating().unwrap();
        });
        task::spawn(async move {
            let sleep_time = 1.0 / temp_output.abs();
            task::sleep(Duration::from_secs(sleep_time as u64)).await;
            turn_off_humidifier().unwrap();
        });
    }
}
