use crate::snap::File;

use std::error::Error;
use std::path::Path;

mod go;
mod rust;

/// This enum describe the snap version
#[derive(PartialEq)]
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

pub struct Options {
    pub snap_version: Version,
}

/// A Generator is a autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn generate<P: AsRef<Path>>(
        &self,
        snap: File,
        source_path: P,
        options: &Options,
    ) -> Result<File, Box<dyn Error>>;
    fn can_generate<P: AsRef<Path>>(&self, source_path: P) -> bool;
}

/// Generators contains the list of supported snapcraft generator
#[derive(Clone)]
pub enum Generators {
    Rust(rust::RustGenerator),
    Go(go::GoGenerator),
}

impl Generator for Generators {
    fn generate<P: AsRef<Path>>(
        &self,
        snap: File,
        source_path: P,
        options: &Options,
    ) -> Result<File, Box<dyn Error>> {
        match *self {
            Generators::Rust(ref generator) => generator.generate(snap, source_path, options),
            Generators::Go(ref generator) => generator.generate(snap, source_path, options),
        }
    }

    fn can_generate<P: AsRef<Path>>(&self, source_path: P) -> bool {
        match *self {
            Generators::Rust(ref generator) => generator.can_generate(&source_path),
            Generators::Go(ref generator) => generator.can_generate(&source_path),
        }
    }
}

/// The GeneratorBuilder allow to return specific generator
/// based on project language
pub struct GeneratorBuilder {
    generators: Vec<Generators>,
}

impl Default for GeneratorBuilder {
    fn default() -> Self {
        let mut generators: Vec<Generators> = Vec::new();
        generators.push(Generators::Rust(rust::RustGenerator {}));
        generators.push(Generators::Go(go::GoGenerator {}));
        GeneratorBuilder { generators }
    }
}

impl GeneratorBuilder {
    pub fn get<P: AsRef<Path>>(&self, source_path: P) -> Result<Generators, Box<dyn Error>> {
        for generator in &self.generators {
            if generator.can_generate(&source_path) {
                return Ok(generator.clone());
            }
        }

        Err("no matching generator found".into())
    }
}
