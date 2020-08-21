use crate::snap::{App, File, Part};
use std::collections::BTreeMap;
use std::error::Error;

/// A Generator is a autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn generate(&self, snap: &File) -> Result<File, Box<dyn Error>>;
}

/// Generators contains the list of supported snapcraft generator
pub enum Generators {
    Rust(RustGenerator),
}

impl Generator for Generators {
    fn generate(&self, snap: &File) -> Result<File, Box<dyn Error>> {
        match *self {
            Generators::Rust(ref generator) => generator.generate(snap),
        }
    }
}

/// The GeneratorBuilder allow to return specific generator
/// based on project language
pub struct GeneratorBuilder {}

impl GeneratorBuilder {
    pub fn get() -> Generators {
        Generators::Rust(RustGenerator {})
    }
}

/// The RustGenerator provide autosnap capability for Rust project
pub struct RustGenerator {}

impl Generator for RustGenerator {
    fn generate(&self, snap: &File) -> Result<File, Box<dyn Error>> {
        let mut snap = snap.clone();

        // generate parts
        let mut parts: BTreeMap<String, Part> = BTreeMap::new();
        parts.insert(
            snap.name.clone(),
            Part {
                plugin: "rust".to_string(),
                source: ".".to_string(),
                build_packages: vec!["libc6-dev".to_string()],
            },
        );
        snap.parts = parts;

        // generate apps
        let mut apps: BTreeMap<String, App> = BTreeMap::new();
        apps.insert(
            snap.name.clone(),
            App {
                command: format!("bin/{}", snap.name),
                plugs: vec![], // TODO
            },
        );
        snap.apps = apps;

        Ok(snap)
    }
}
