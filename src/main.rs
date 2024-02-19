use rocket_cors::{AllowedOrigins, CorsOptions};
use std::thread;

#[macro_use]
extern crate rocket;
extern crate engiffen;
extern crate rocket_cors;
mod route {
    pub mod heartbeat;
    pub mod index;
    pub mod project;
    pub mod sensor_value;
    pub mod webcam;
}

pub mod service {
    pub mod database;
    pub mod gpio;
    pub mod sensor;
    pub mod webcam;
}

pub mod basic_runners {
    pub mod manage_climate;
    pub mod sensor_logger;
}

#[launch]
async fn rocket() -> _ {
    thread::spawn(|| basic_runners::sensor_logger::entry_loop());
    thread::spawn(|| basic_runners::manage_climate::entry_loop());
    let mut index_routes = routes![route::index::index, route::index::files];
    index_routes[1].rank = 2;
    let cors = CorsOptions::default().allowed_origins(AllowedOrigins::all());

    rocket::build()
        .mount("/", index_routes)
        .mount("/heartbeat", routes![route::heartbeat::get])
        .mount("/webcam", routes![route::webcam::gif])
        .mount(
            "/project",
            routes![
                route::project::all_projects,
                route::project::get_project,
                route::project::create,
                route::project::update,
                route::project::delete,
                route::project::start,
                route::project::end,
                route::project::set_settings
            ],
        )
        .mount(
            "/sensor",
            routes![route::sensor_value::get, route::sensor_value::get_historic],
        )
        .attach(cors.to_cors().unwrap())
}
