use crate::snap::{App, File, Part};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::path::Path;

/// A Generator is a autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn generate<P: AsRef<Path>>(&self, snap: &File, repo_path: P) -> Result<File, Box<dyn Error>>;
}

/// Generators contains the list of supported snapcraft generator
#[derive(Clone)]
pub enum Generators {
    Rust(RustGenerator),
    Go(GoGenerator),
}

impl Generator for Generators {
    fn generate<P: AsRef<Path>>(&self, snap: &File, repo_path: P) -> Result<File, Box<dyn Error>> {
        match *self {
            Generators::Rust(ref generator) => generator.generate(snap, repo_path),
            Generators::Go(ref generator) => generator.generate(snap, repo_path),
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
        generators.insert("Cargo.toml".to_string(), Generators::Rust(RustGenerator {}));
        generators.insert("go.mod".to_string(), Generators::Go(GoGenerator {}));

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

/// The RustGenerator provide autosnap capability for Rust project
#[derive(Clone)]
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
                build_packages: Some(vec!["libc6-dev".to_string()]),
                stage_packages: None,
                go_import_path: None,
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
                    plugs: None, // TODO
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
                            plugs: None,
                        },
                    );
                }
            }
        }
        snap.apps = apps;

        Ok(snap)
    }
}

/// The RustGenerator provide autosnap capability for Go project
#[derive(Clone)]
pub struct GoGenerator {}

impl Generator for GoGenerator {
    fn generate<P: AsRef<Path>>(&self, snap: &File, repo_path: P) -> Result<File, Box<dyn Error>> {
        let mut snap = snap.clone();

        // fetch go import path from go.mod
        let mod_file = fs::read_to_string(repo_path.as_ref().join("go.mod"))?;
        let line_end = mod_file.chars().position(|s| s == '\n').unwrap();
        let import_path = &mod_file[..line_end].replace("module ", "");

        // generate parts
        let mut parts: BTreeMap<String, Part> = BTreeMap::new();
        parts.insert(
            snap.name.clone(),
            Part {
                plugin: "go".to_string(),
                source: ".".to_string(),
                build_packages: Some(vec!["gcc".to_string(), "libc6-dev".to_string()]),
                stage_packages: None,
                go_import_path: Some(import_path.to_string()),
            },
        );
        snap.parts = parts;

        Ok(snap)
    }
}
