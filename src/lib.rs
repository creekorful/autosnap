use crate::generator::{Generators, Options};
use crate::snap::{File, SNAPCRAFT_YAML};

use core::result;
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use url::Url;

pub mod generator;
pub mod snap;

type Result<T> = result::Result<T, Box<dyn Error>>;

/// Fetch given remote source and 'install' it in the working directory,
/// and return path to the source.
///
/// ```no_run
/// use autosnap::fetch_source;
/// use url::Url;
/// let source = fetch_source(&Url::parse("https://github.com/creekorful/osync.git").unwrap()).unwrap();
/// ```
pub fn fetch_source(source_url: &Url) -> Result<PathBuf> {
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

/// Package source located at given path using given options
///
/// ```no_run
/// use autosnap::{fetch_source, package_source};
/// use url::Url;
/// use autosnap::generator::{Options, Version};
/// let path = fetch_source(&Url::parse("https://www.github.com/creekorful/osync.git").unwrap()).unwrap();
/// let snap = package_source(&path, &Options {source_name: "".to_string(), snap_version: Version::Git}).unwrap();
/// ```
pub fn package_source<P: AsRef<Path>>(source_path: P, options: &Options) -> Result<File> {
    // convert . into current dir
    let source_path = if source_path.as_ref().eq(Path::new(".")) {
        env::current_dir()?
    } else {
        source_path.as_ref().to_path_buf()
    };

    let mut options = options.clone();
    let source_name = source_path.file_name().unwrap().to_str().unwrap();
    options.source_name = source_name.to_string();

    // Determinate if not already packaged
    if source_path.join(SNAPCRAFT_YAML).exists()
        || source_path.join("snap").join(SNAPCRAFT_YAML).exists()
    {
        return Err(format!("{} is already packaged", source_name).into());
    }

    // Use appropriate generator to complete the generation
    Generators::generate(&source_path, &options)
}
