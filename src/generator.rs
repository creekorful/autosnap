use crate::snap::{App, File, Part};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

/// A Generator is a autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn generate<P: AsRef<Path>>(&self, snap: &File, repo_path: P) -> Result<File, Box<dyn Error>>;
}

/// Generators contains the list of supported snapcraft generator
pub enum Generators {
    Rust(RustGenerator),
}

impl Generator for Generators {
    fn generate<P: AsRef<Path>>(&self, snap: &File, repo_path: P) -> Result<File, Box<dyn Error>> {
        match *self {
            Generators::Rust(ref generator) => generator.generate(snap, repo_path),
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
    fn generate<P: AsRef<Path>>(&self, snap: &File, repo_path: P) -> Result<File, Box<dyn Error>> {
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

        // crate can contains either a single binary if there's a src/main.rs
        // otherwise it will contains as many binaries as there is file matching src/bin/*.rs
        // TODO support multiple crates project?

        if repo_path.as_ref().join("src").join("main.rs").exists() {
            apps.insert(
                snap.name.clone(),
                App {
                    command: format!("bin/{}", snap.name),
                    plugs: vec![], // TODO
                },
            );
        } else {
            for entry in fs::read_dir(repo_path.as_ref().join("src").join("bin"))? {
                let entry = entry?;

                let file_name = entry.file_name().to_str().unwrap().to_string();
                if entry.path().is_file() && file_name.ends_with(".rs") {
                    let binary_name = file_name.replace(".rs", "");
                    apps.insert(
                        binary_name.clone(),
                        App {
                            command: format!("bin/{}", binary_name),
                            plugs: vec![], // TODO
                        },
                    );
                }
            }
        }

        snap.apps = apps;

        Ok(snap)
    }
}
