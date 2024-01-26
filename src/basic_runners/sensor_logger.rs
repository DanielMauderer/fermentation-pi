use std::thread;

use crate::service::{
    database::{
        project::get_active_project,
        sensor::{add_datapoint, HistoricSensorData},
    },
    sensor::get_sensor_data,
};
use nokhwa::{Camera, CameraFormat, FrameFormat};

pub fn entry_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut camera = match Camera::new(
        0,                                                                // index
        Some(CameraFormat::new_from(1920, 1080, FrameFormat::MJPEG, 30)), // format
    ) {
        Ok(camera) => camera,
        Err(e) => {
            println!("Error: {}", e);
            return Err(Box::from(e));
        }
    };
    // open stream
    match camera.open_stream() {
        Ok(_) => {}
        Err(e) => {
            println!("Error: {}", e);
            return Err(Box::from(e));
        }
    };
    loop {
        take_sensor_data()?;
        let _ = take_webcam_image(&mut camera)?;
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
            println!("Error: {}", e);
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
            println!("Error: {}", e);
            return Err(Box::from(e));
        }
    }
}
