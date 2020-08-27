use std::error::Error;
use std::path::Path;

use crate::snap::File;

mod go;
mod rust;

/// A Generator is a autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn generate<P: AsRef<Path>>(&self, snap: File, source_path: P) -> Result<File, Box<dyn Error>>;
    fn can_generate<P: AsRef<Path>>(&self, source_path: P) -> bool;
}

/// Generators contains the list of supported snapcraft generator
#[derive(Clone)]
pub enum Generators {
    Rust(rust::RustGenerator),
    Go(go::GoGenerator),
}

impl Generator for Generators {
    fn generate<P: AsRef<Path>>(&self, snap: File, source_path: P) -> Result<File, Box<dyn Error>> {
        match *self {
            Generators::Rust(ref generator) => generator.generate(snap, source_path),
            Generators::Go(ref generator) => generator.generate(snap, source_path),
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
