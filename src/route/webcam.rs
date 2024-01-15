use rocket::fs::NamedFile;

use crate::service::webcam::generate_gif;

#[get("/gif/<project>")]
pub async fn gif(project: u32) -> Option<NamedFile> {
    generate_gif(project).await
}
