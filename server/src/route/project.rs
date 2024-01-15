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
    create_new_project, delete_project, end_project, read_project, read_projects, start_project,
    update_project, Project,
};
use ::serde::Deserialize;
use rocket::serde::json::Json;

#[get("/")]
pub fn all_projects() -> Json<Vec<Project>> {
    Json(read_projects())
}

#[get("/<id>")]
pub fn get_project(id: u32) -> Json<Project> {
    Json(read_project(id))
}

#[post("/", format = "json", data = "<project>")]
pub fn create(project: Json<CreateRequest>) -> Json<Project> {
    let project = create_new_project(project.name.clone(), project.description.clone());
    Json(project)
}

#[put("/<id>", format = "json", data = "<project>")]
pub fn update(id: u32, project: Json<PutRequest>) -> Json<Project> {
    update_project(id, project.name.clone(), project.description.clone());
    Json(read_project(id))
}

#[delete("/<id>")]
pub fn delete(id: u32) {
    delete_project(id);
}

#[post("/<id>/start")]
pub fn start(id: u32) {
    let start_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    start_project(id, start_at);
}

#[post("/<id>/end")]
pub fn end(id: u32) {
    let endend_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    end_project(id, endend_at);
}
