use std::thread;

pub fn entry_loop() {
    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
