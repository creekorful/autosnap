#[macro_use]
extern crate log;
extern crate simple_logger;

mod generator;

use crate::generator::{Generator, GeneratorBuilder};
use crate::snap::SNAPCRAFT_YAML;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use url::Url;

pub mod snap;

pub fn fetch_source(source_url: &Url) -> Result<PathBuf, Box<dyn Error>> {
    // TODO support tarball etc

    let cwd = env::current_dir()?;
    let source_name = source_url.path_segments().unwrap().last().unwrap();
    let path = cwd.join(source_name);

    // Clone the source code
    git2::Repository::clone(source_url.as_str(), &path)?;

    Ok(path)
}

pub fn package_source<P: AsRef<Path>>(source_path: P) -> Result<snap::File, Box<dyn Error>> {
    let source_name = source_path.as_ref().file_name().unwrap().to_str().unwrap();

    // Determinate if not already packaged
    if source_path.as_ref().join(SNAPCRAFT_YAML).exists()
        || source_path
            .as_ref()
            .join("snap")
            .join(SNAPCRAFT_YAML)
            .exists()
    {
        return Err(format!("{} is already packaged", source_name).into());
    }

    // TODO Identify the project license
    // using https://github.com/jpeddicord/askalono

    // Create snap with defaults set
    let snap = snap::File::new(source_name);

    // And use appropriate generator to complete the generation
    let generator_builder = GeneratorBuilder::default();
    let generator = match generator_builder.get(&source_path) {
        Ok(generator) => generator,
        Err(e) => {
            return Err(e);
        }
    };

    generator.generate(&snap, &source_path)
}
