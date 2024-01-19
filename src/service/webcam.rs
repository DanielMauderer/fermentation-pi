use std::{fs::File, path::Path};

use glob::glob;

use engiffen::{engiffen, load_images, Gif};
use rocket::fs::NamedFile;

pub async fn generate_gif(project: u32) -> Result<NamedFile, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();

    let files = match glob(format!("./webcam/{project}/*.jpg").as_str()) {
        Ok(files) => files,
        Err(e) => return Err(Box::from(e)),
    };
    for entry in files {
        match entry {
            Ok(path) => paths.push(path),
            Err(e) => return Err(Box::from(e)),
        }
    }

    let images = load_images(&paths);
    let gif: Gif = engiffen(&images, 5, engiffen::Quantizer::NeuQuant(10))?;
    let mut output = File::create("tmp/output.gif")?;

    let _ = gif.write(&mut output);
    Ok(NamedFile::open(Path::new("tmp/output.gif")).await?)
}
