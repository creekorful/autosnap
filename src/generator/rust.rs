use std::path::Path;
use crate::snap::{File, Part, App};
use std::error::Error;
use cargo_lock::Lockfile;
use std::collections::BTreeMap;
use std::fs;
use crate::generator::Generator;

const CARGO_LOCK: &str = "Cargo.lock";
const LIBSSL_DEV: &str = "libssl-dev";

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
                        && !build_packages.iter().any(|p| p == LIBSSL_DEV)
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
