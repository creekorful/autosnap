use crate::generator::{Generator, GeneratorBuilder, Options, Version};
use crate::snap::SNAPCRAFT_YAML;

use askalono::{Store, TextData};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use url::Url;

pub mod generator;
pub mod snap;

static LICENSE_CACHE: &[u8] = include_bytes!("embedded-cache.bin.zstd");

pub fn fetch_source(source_url: &Url) -> Result<PathBuf, Box<dyn Error>> {
    // TODO support tarball etc

    let cwd = env::current_dir()?;
    let source_name = source_url
        .path_segments()
        .unwrap()
        .last()
        .unwrap()
        .replace(".git", "");
    let path = cwd.join(source_name);

    // Clone the source code
    git2::Repository::clone(source_url.as_str(), &path)?;

    Ok(path)
}

pub fn package_source<P: AsRef<Path>>(
    source_path: P,
    options: &Options,
) -> Result<snap::File, Box<dyn Error>> {
    // convert . into current dir
    let source_path = if source_path.as_ref().eq(Path::new(".")) {
        env::current_dir()?
    } else {
        source_path.as_ref().to_path_buf()
    };

    let source_name = source_path.file_name().unwrap().to_str().unwrap();

    // Determinate if not already packaged
    if source_path.join(SNAPCRAFT_YAML).exists()
        || source_path.join("snap").join(SNAPCRAFT_YAML).exists()
    {
        return Err(format!("{} is already packaged", source_name).into());
    }

    // TODO Identify the project license
    // using https://github.com/jpeddicord/askalono

    // Create snap with defaults set
    let mut snap = snap::File::new(source_name);

    match &options.snap_version {
        Version::Git => snap.version = "git".to_string(),
        Version::Fixed(version) => {
            log::debug!("Set snap version to {}", version);
            snap.version = version.clone()
        }
        _ => {}
    }

    // Try to autodetect license if possible
    if let Some((license, filename)) = find_license(&source_path)? {
        let store = Store::from_cache(LICENSE_CACHE)?;
        let result = store.analyze(&TextData::from(license));

        // TODO use real value above
        if result.score > 0.9 {
            snap.license = result.name.to_string();
            log::debug!(
                "Auto-detect snap license ({}) from file {}",
                result.name,
                filename
            );
        }
    }

    // And use appropriate generator to complete the generation
    let generator_builder = GeneratorBuilder::default();
    let generator = match generator_builder.get(&source_path) {
        Ok(generator) => generator,
        Err(e) => {
            return Err(e);
        }
    };

    generator.generate(snap, &source_path, &options)
}

fn find_license<P: AsRef<Path>>(source_path: P) -> Result<Option<(String, String)>, io::Error> {
    if source_path.as_ref().join("LICENSE").exists() {
        fs::read_to_string(source_path.as_ref().join("LICENSE"))
            .map(|v| Some((v, "LICENSE".to_string())))
    } else if source_path.as_ref().join("LICENSE.md").exists() {
        fs::read_to_string(source_path.as_ref().join("LICENSE.md"))
            .map(|v| Some((v, "LICENSE.md".to_string())))
    } else if source_path.as_ref().join("COPYING").exists() {
        fs::read_to_string(source_path.as_ref().join("COPYING"))
            .map(|v| Some((v, "COPYING".to_string())))
    } else {
        Ok(None)
    }
}
