use async_std::task;
use pid::Pid;
use std::{thread, time::Duration};

use crate::service::{
    database::{project::get_active_project, sensor::SensorData},
    gpio::{turn_off_heating, turn_off_humidifier, turn_on_heating, turn_on_humidifier},
    sensor::get_sensor_data,
};

pub async fn entry_loop() -> Result<(), Box<dyn std::error::Error>> {
    let project = get_active_project()?;
    let mut hum_pid: Pid<f32> = Pid::new(project.settings.hum, 100.0);
    hum_pid.p(10.0, 100.0).i(4.5, 100.0).d(0.25, 100.0);
    let mut temp_pid: Pid<f32> = Pid::new(project.settings.temp, 100.0);
    temp_pid.p(10.0, 100.0).i(4.5, 100.0).d(0.25, 100.0);
    warn!("1");
    let mut sensor_data: SensorData = get_sensor_data().await?;
    warn!("1");

    loop {
        let sensor_data_task = get_sensor_data();
        warn!("started task");
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
        warn!("awaiting task");

        sensor_data = sensor_data_task.await?;
        warn!("awaiting task done");
    }
}
