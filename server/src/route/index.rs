use rocket::fs::NamedFile;
use std::io;
use std::path::{Path, PathBuf};

#[get("/")]
pub async fn index() -> io::Result<NamedFile> {
    NamedFile::open("fermentation-pi-client/dist/index.html").await
}

#[get("/<file..>")]
pub async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("fermentation-pi-client/dist/").join(file))
        .await
        .ok()
}
