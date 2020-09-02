use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, io};

use askalono::{Store, TextData};

use crate::generator::go::GoProvider;
use crate::generator::gradle::GradleProvider;
use crate::generator::python::PythonProvider;
use crate::generator::rust::RustProvider;
use crate::snap::{App, File, Part};
use crate::Result;

mod go;
mod gradle;
mod python;
mod rust;

static LICENSE_CACHE: &[u8] = include_bytes!("embedded-cache.bin.zstd");

/// This enum describe the snap version strategy:
/// i.e how the snap version will be set.
#[derive(PartialEq, Clone)]
pub enum Version {
    /// Version will be set to "git" i.e Snapcraft will set snap version using git
    Git,
    /// Version will be set by autosnap if possible (i.e by trying to parse Cargo.toml, etc...)
    Auto,
    /// Version will be set as provided
    Fixed(String),
}

/// Convert a string into his Version representation
impl From<&str> for Version {
    fn from(s: &str) -> Self {
        match s {
            "git" => Version::Git,
            "auto" => Version::Auto,
            version => Version::Fixed(version.to_string()),
        }
    }
}

/// The generator configuration
#[derive(Clone)]
pub struct Options {
    /// The snap version strategy
    pub snap_version: Version,
    pub source_name: String,
}

trait Provider<G: Generator> {
    fn provide<P: AsRef<Path>>(source_path: P, source_name: &str) -> Result<G>;
    fn can_provide<P: AsRef<Path>>(source_path: P) -> bool;
}

/// A `Generator` is an Autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn name(&self) -> Result<Option<String>>;
    fn version(&self) -> Result<Option<String>>;
    fn summary(&self) -> Result<Option<String>>;
    fn description(&self) -> Result<Option<String>>;
    fn license(&self) -> Result<Option<String>>;
    fn parts(&self) -> Result<BTreeMap<String, Part>>;
    fn apps(&self) -> Result<BTreeMap<String, App>>;
}

/// The list of supported generators
pub enum Generators {
    Go(GoProvider),
    Rust(RustProvider),
    Python(PythonProvider),
    Gradle(GradleProvider),
}

impl Generators {
    /// Find the generator that can package given source
    fn find_generator<P: AsRef<Path>>(
        source_path: P,
        source_name: &str,
    ) -> Result<Box<dyn Generator>> {
        // TODO improve below
        if RustProvider::can_provide(&source_path) {
            log::debug!("Using RustGenerator");
            let provider = RustProvider::provide(&source_path, source_name);
            match provider {
                Ok(v) => Ok(Box::new(v)),
                Err(e) => Err(e),
            }
        } else if GoProvider::can_provide(&source_path) {
            log::debug!("Using GoGenerator");
            let provider = GoProvider::provide(&source_path, source_name);
            match provider {
                Ok(v) => Ok(Box::new(v)),
                Err(e) => Err(e),
            }
        } else if PythonProvider::can_provide(&source_path) {
            log::debug!("Using PythonGenerator");
            let provider = PythonProvider::provide(&source_path, source_name);
            match provider {
                Ok(v) => Ok(Box::new(v)),
                Err(e) => Err(e),
            }
        } else if GradleProvider::can_provide(&source_path) {
            log::debug!("Using GradleProvider");
            let provider = GradleProvider::provide(&source_path, source_name);
            match provider {
                Ok(v) => Ok(Box::new(v)),
                Err(e) => Err(e),
            }
        } else {
            Err("Cannot find corresponding generator.".into())
        }
    }

    /// Generate the Snap file using source in given directory with given options
    ///
    /// ```no_run
    /// use autosnap::generator::{Generators, Options, Version};
    /// let opts = Options{snap_version: Version::Git, source_name: "source-code".to_string()};
    /// let file = Generators::generate("/tmp/source-code", &opts).unwrap();
    /// ```
    pub fn generate<P: AsRef<Path>>(source_path: P, options: &Options) -> Result<File> {
        let generator = Generators::find_generator(&source_path, &options.source_name)?;

        // Create snap with defaults set
        let mut snap = File::new(&options.source_name);

        // Set snap version as needed
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

        // Use generator to complete Snap
        if let Some(name) = generator.name()? {
            log::debug!("Set snap name to `{}`", name);
            snap.name = name;
        }
        // Delegate version detection to Generator
        if options.snap_version == Version::Auto {
            if let Some(version) = generator.version()? {
                log::debug!("Set snap version to `{}`", version);
                snap.version = version;
            }
        }
        if let Some(summary) = generator.summary()? {
            log::debug!("Set snap summary to `{}`", summary);
            snap.summary = summary;
        }
        if let Some(description) = generator.description()? {
            log::debug!("Set snap description to `{}`", description);
            snap.description = description;
        }
        if let Some(license) = generator.license()? {
            log::debug!("Set snap license to `{}`", license);
            snap.license = license;
        }

        let parts = generator.parts()?;
        if parts.is_empty() {
            return Err("No parts found.".into());
        }
        snap.parts = parts;

        let apps = generator.apps()?;
        if apps.is_empty() {
            return Err("No apps found.".into());
        }
        snap.apps = apps;

        Ok(snap)
    }
}

/// Find the source license file. This naive method will try to find
/// the project license file.
fn find_license<P: AsRef<Path>>(source_path: P) -> io::Result<Option<(String, String)>> {
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
