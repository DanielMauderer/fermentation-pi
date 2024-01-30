use std::time::SystemTime;

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateRequest {
    name: String,
    description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PutRequest {
    name: Option<String>,
    description: Option<String>,
}

use crate::service::database::project::{
    create_new_project, delete_project, end_project, read_project, read_projects,
    set_project_settings, start_project, update_project, Project, Settings,
};
use ::serde::Deserialize;
use rocket::serde::json::Json;

#[get("/")]
pub fn all_projects() -> Json<Option<Vec<Project>>> {
    Json(match read_projects() {
        Ok(projects) => Some(projects),
        Err(_) => None,
    })
}

#[get("/<id>")]
pub fn get_project(id: u32) -> Json<Option<Project>> {
    Json(match read_project(id) {
        Ok(project) => Some(project),
        Err(_) => None,
    })
}

#[post("/", format = "json", data = "<project>")]
pub fn create(project: Json<CreateRequest>) {
    let _ = create_new_project(project.name.clone(), project.description.clone());
}

#[put("/<id>", format = "json", data = "<project>")]
pub fn update(id: u32, project: Json<PutRequest>) -> Json<Option<Project>> {
    let _ = update_project(id, project.name.clone(), project.description.clone());
    Json(match read_project(id) {
        Ok(project) => Some(project),
        Err(_) => None,
    })
}

#[delete("/<id>")]
pub fn delete(id: u32) {
    let _ = delete_project(id);
}

#[post("/<id>/start")]
pub fn start(id: u32) {
    let _ = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(start_at) => start_project(id, start_at.as_secs()),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
}

#[post("/<id>/end")]
pub fn end(id: u32) {
    let _ = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(end_at) => end_project(id, end_at.as_secs()),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
}

#[post("/<id>/settings", format = "json", data = "<settings>")]
pub fn set_settings(id: u32, settings: Json<Settings>) {
    let _ = set_project_settings(id, settings.0);
}
