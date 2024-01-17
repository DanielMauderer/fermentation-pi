use std::thread;

use crate::service::{
    database::sensor::{add_datapoint, HistoricSensorData, SensorData},
    sensor::{get_humidity, get_temperature},
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
        take_sensor_data();
        take_webcam_image(&mut camera);
        thread::sleep(std::time::Duration::from_secs(60));
    }
}

fn take_sensor_data() {
    let data = HistoricSensorData {
        time: chrono::Utc::now().timestamp() as u64,
        data: SensorData {
            temp: get_temperature(),
            hum: get_humidity(),
        },
    };
    add_datapoint(data);
}

fn take_webcam_image(camera: &mut Camera) -> Result<(), Box<dyn std::error::Error>> {
    let frame = match camera.frame() {
        Ok(frame) => frame,
        Err(e) => {
            println!("Error: {}", e);
            return Err(Box::from(e));
        }
    };
    let path = format!("./webcam/0/{}.jpg", chrono::Utc::now().timestamp());
    match frame.save_with_format(path, image::ImageFormat::Jpeg) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {}", e);
            return Err(Box::from(e));
        }
    }
}
