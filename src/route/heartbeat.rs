#[get("/")]
pub fn get() -> &'static str {
    "Hello, world!"
}
