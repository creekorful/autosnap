use crate::snap::{App, File, Part};
use cargo_lock::Lockfile;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::path::Path;

const CARGO_LOCK: &str = "Cargo.lock";
const LIBSSL_DEV: &str = "libssl-dev";

/// A Generator is a autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn generate<P: AsRef<Path>>(&self, snap: &File, source_path: P)
        -> Result<File, Box<dyn Error>>;
}

/// Generators contains the list of supported snapcraft generator
#[derive(Clone)]
pub enum Generators {
    Rust(RustGenerator),
    Go(GoGenerator),
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
        generators.insert("Cargo.toml".to_string(), Generators::Rust(RustGenerator {}));
        //generators.insert("go.mod".to_string(), Generators::Go(GoGenerator {}));

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
    fn generate<P: AsRef<Path>>(
        &self,
        snap: &File,
        source_path: P,
    ) -> Result<File, Box<dyn Error>> {
        let mut snap = snap.clone();

        // generate parts
        let mut build_packages = vec!["libc6-dev".to_string()];

        // Determinate custom build packages based on cargo.lock
        if source_path.as_ref().join(CARGO_LOCK).exists() {
            let lock_file = Lockfile::load(source_path.as_ref().join(CARGO_LOCK))?;

            for package in lock_file.packages {
                for dependency in package.dependencies {
                    if dependency.name.as_str().starts_with("openssl-")
                        && !build_packages.contains(&LIBSSL_DEV.to_string()) // TODO improve
                    {
                        // watch out for openssl-sys dependencies
                        debug!(
                            "Adding {} build package as required by {}",
                            LIBSSL_DEV, package.name
                        );
                        build_packages.push(LIBSSL_DEV.to_string());
                    }
                }
            }
        }

        // generate parts
        let mut parts: BTreeMap<String, Part> = BTreeMap::new();
        parts.insert(
            snap.name.clone(),
            Part {
                plugin: "rust".to_string(),
                source: ".".to_string(),
                build_packages: Option::from(build_packages),
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

        if source_path.as_ref().join("src").join("main.rs").exists() {
            debug!("found single executable (name: {})", snap.name);
            apps.insert(
                snap.name.clone(),
                App {
                    command: format!("bin/{}", snap.name),
                    plugs: None, // TODO
                },
            );
        } else {
            for entry in fs::read_dir(source_path.as_ref().join("src").join("bin"))? {
                let entry = entry?;

                let file_name = entry.file_name().to_str().unwrap().to_string();
                if entry.path().is_file() && file_name.ends_with(".rs") {
                    let binary_name = file_name.replace(".rs", "");
                    debug!("found executable (name: {})", binary_name);
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
    fn generate<P: AsRef<Path>>(
        &self,
        snap: &File,
        source_path: P,
    ) -> Result<File, Box<dyn Error>> {
        let mut snap = snap.clone();

        // fetch go import path from go.mod
        let mod_file = fs::read_to_string(source_path.as_ref().join("go.mod"))?;
        let line_end = mod_file.chars().position(|s| s == '\n').unwrap();
        let import_path = &mod_file[..line_end].replace("module ", "");
        debug!("setting go-import-path to {} (using go.mod)", import_path);

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

        // TODO find apps (func main() in package main)

        Ok(snap)
    }
}
