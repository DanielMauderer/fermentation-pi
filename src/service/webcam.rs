use std::{env, path::Path};

use glob::glob;

use rocket::fs::NamedFile;
use std::process::Command;

pub async fn generate_gif(project: u32) -> Result<NamedFile, Box<dyn std::error::Error>> {
    let dir = env::current_dir()?;
    let root = dir.to_str().unwrap();
    let files = glob(format!("./webcam/{project}/*.png").as_str())?;
    let mut command = Command::new("gifski");
    command.arg("-o").arg(format!("{}/tmp/output.gif", root));
    for path in files {
        let path = path?;
        let path = path.to_str().unwrap();
        command.arg(format!("{}", path));
    }
    //let fps = files.count() / 30;
    command.spawn()?;
    Ok(NamedFile::open(Path::new("tmp/output.gif")).await?)
}
