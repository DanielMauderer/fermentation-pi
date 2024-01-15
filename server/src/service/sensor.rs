use rand::Rng;

pub fn get_temperature() -> f32 {
    let mut rng = rand::thread_rng();

    return rng.gen_range(0.0..40.0);
}

pub fn get_humidity() -> f32 {
    let mut rng = rand::thread_rng();

    return rng.gen_range(0.0..100.0);
}
