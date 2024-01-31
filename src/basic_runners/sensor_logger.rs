use std::thread;

use crate::service::{
    database::{
        project::get_active_project,
        sensor::{add_datapoint, HistoricSensorData},
    },
    sensor::get_sensor_data,
};
use nokhwa::{Camera, CameraFormat, FrameFormat};

pub fn entry_loop() {
    let mut camera = match Camera::new(
        0,
        Some(CameraFormat::new_from(1920, 1080, FrameFormat::MJPEG, 30)), // format
    ) {
        Ok(camera) => camera,
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };
    // open stream
    match camera.open_stream() {
        Ok(_) => {}
        Err(e) => {
            error!("Error: {}", e);
            return;
        }
    };
    loop {
        let sensor_task = take_sensor_data();
        let webcam_error = take_webcam_image(&mut camera).err();
        match sensor_task {
            Ok(_) => {}
            Err(e) => {
                error!("Error: {}", e);
                return;
            }
        }
        match webcam_error {
            Some(e) => {
                error!("Error: {}", e);
                return;
            }
            None => {}
        }
        thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn take_sensor_data() -> Result<(), Box<dyn std::error::Error>> {
    let data = HistoricSensorData {
        time: chrono::Utc::now().timestamp() as u64,
        data: get_sensor_data()?,
    };
    add_datapoint(data)
}

fn take_webcam_image(camera: &mut Camera) -> Result<(), Box<dyn std::error::Error>> {
    let frame = match camera.frame() {
        Ok(frame) => frame,
        Err(e) => {
            error!("Error: {}", e);
            return Err(Box::from(e));
        }
    };
    let path = format!(
        "./webcam/{}/{}.png",
        get_active_project()?.id,
        chrono::Utc::now().timestamp()
    );
    match frame.save_with_format(path, image::ImageFormat::Png) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error: {}", e);
            return Err(Box::from(e));
        }
    }
}
