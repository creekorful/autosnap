use crate::snap::File;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

mod go;
mod rust;

/// A Generator is a autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn generate<P: AsRef<Path>>(&self, snap: &File, source_path: P)
        -> Result<File, Box<dyn Error>>;
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
        snap: &File,
        source_path: P,
    ) -> Result<File, Box<dyn Error>> {
        match *self {
            Generators::Rust(ref generator) => generator.generate(snap, source_path),
            Generators::Go(ref generator) => generator.generate(snap, source_path),
        }
    }
}

/// The GeneratorBuilder allow to return specific generator
/// based on project language
pub struct GeneratorBuilder {
    generators: HashMap<String, Generators>,
}

impl Default for GeneratorBuilder {
    fn default() -> Self {
        let mut generators: HashMap<String, Generators> = HashMap::new();

        // Fill supported languages here
        generators.insert(
            "Cargo.toml".to_string(),
            Generators::Rust(rust::RustGenerator {}),
        );
        generators.insert("go.mod".to_string(), Generators::Go(go::GoGenerator {})); // TODO better?

        GeneratorBuilder { generators }
    }
}

impl GeneratorBuilder {
    pub fn get<P: AsRef<Path>>(&self, path: P) -> Result<Generators, Box<dyn Error>> {
        for (extension, generator) in &self.generators {
            for entry in fs::read_dir(&path)? {
                let entry = entry?;

                let file_name = entry.file_name().to_str().unwrap().to_string();
                if entry.path().is_file() && file_name.ends_with(extension) {
                    return Ok(generator.clone());
                }
            }
        }

        Err("no matching generator found".into())
    }
}
