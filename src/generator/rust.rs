use crate::generator::{Generator, Provider};
use crate::snap::{App, Part};
use crate::Result;
use cargo_lock::Lockfile;
use cargo_toml::Manifest;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct RustGenerator {
    cargo_toml: Manifest,
    cargo_lock: Option<Lockfile>,
    source_path: PathBuf,
    source_name: String,
}

pub struct RustProvider {}

impl Provider<RustGenerator> for RustProvider {
    fn provide<P: AsRef<Path>>(source_path: P, source_name: &str) -> Result<RustGenerator> {
        let manifest = Manifest::from_path(source_path.as_ref().join("Cargo.toml"))?;
        let lockfile = if source_path.as_ref().join("Cargo.lock").exists() {
            Some(Lockfile::load(source_path.as_ref().join("Cargo.lock"))?)
        } else {
            None
        };

        Ok(RustGenerator {
            cargo_toml: manifest,
            cargo_lock: lockfile,
            source_path: source_path.as_ref().to_path_buf(),
            source_name: source_name.to_string(),
        })
    }

    fn can_provide<P: AsRef<Path>>(source_path: P) -> bool {
        source_path.as_ref().join("Cargo.toml").exists()
    }
}

impl Generator for RustGenerator {
    fn name(&self) -> Result<Option<String>> {
        Ok(self.cargo_toml.package.as_ref().cloned().map(|p| p.name))
    }

    fn version(&self) -> Result<Option<String>> {
        Ok(self.cargo_toml.package.as_ref().cloned().map(|p| p.version))
    }

    fn summary(&self) -> Result<Option<String>> {
        Ok(self
            .cargo_toml
            .package
            .as_ref()
            .cloned()
            .and_then(|p| p.description))
    }

    fn description(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn license(&self) -> Result<Option<String>> {
        Ok(self
            .cargo_toml
            .package
            .as_ref()
            .cloned()
            .and_then(|p| p.license))
    }

    fn parts(&self) -> Result<BTreeMap<String, Part>> {
        let mut parts = BTreeMap::default();

        // Determinate custom build packages based on Cargo.lock
        let build_packages = if let Some(lockfile) = &self.cargo_lock {
            Some(find_build_packages(lockfile))
        } else {
            None
        };

        // Create parts
        parts.insert(
            self.source_name.clone(),
            Part {
                plugin: "rust".to_string(),
                source: ".".to_string(),
                build_packages,
                stage_packages: None,
                go_import_path: None,
                python_version: None,
            },
        );

        Ok(parts)
    }

    fn apps(&self) -> Result<BTreeMap<String, App>> {
        find_apps(self.source_path.clone(), &self.source_name)
    }
}

fn find_build_packages(lockfile: &Lockfile) -> Vec<String> {
    let mut build_packages = vec!["libc6-dev".to_string()];
    for package in &lockfile.packages {
        for dependency in &package.dependencies {
            if dependency.name.as_str().starts_with("openssl-")
                && !build_packages.iter().any(|p| p == "libssl-dev")
            {
                // watch out for openssl-sys dependencies
                log::debug!(
                    "Adding libssl-dev build package as required by {}",
                    package.name
                );
                build_packages.push("libssl-dev".to_string());
            }
        }
    }

    build_packages
}

fn find_apps<P: AsRef<Path>>(source_path: P, source_name: &str) -> Result<BTreeMap<String, App>> {
    let mut apps: BTreeMap<String, App> = BTreeMap::new();

    // crate can contains either a single binary if there's a src/main.rs
    // otherwise it will contains as many binaries as there is file matching src/bin/*.rs
    // TODO support multiple crates project?

    if source_path.as_ref().join("src").join("main.rs").exists() {
        log::debug!("Found single executable (name: {})", source_name);
        apps.insert(
            source_name.to_string(),
            App {
                command: format!("bin/{}", source_name),
                plugs: None, // TODO
            },
        );
    } else {
        for entry in fs::read_dir(source_path.as_ref().join("src").join("bin"))? {
            let entry = entry?;

            let file_name = entry.file_name().to_str().unwrap().to_string();
            if entry.path().is_file() && file_name.ends_with(".rs") {
                let binary_name = file_name.replace(".rs", "");
                log::debug!("Found executable (name: {})", binary_name);
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

    Ok(apps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_lock::{Dependency, Name, Package, Version};
    use std::str::FromStr;
    use tempdir::TempDir;

    #[test]
    fn test_find_apps_single_app() {
        let tmp_dir = TempDir::new("autosnap").expect("unable to create temporary dir");
        fs::create_dir_all(tmp_dir.path().join("src")).expect("unable to create src");

        fs::write(tmp_dir.path().join("src").join("main.rs"), "fn main() {}")
            .expect("unable to write src/main.rs");

        let apps = find_apps(tmp_dir, "autosnap").expect("unable to find apps");
        assert!(apps.contains_key("autosnap"));

        let app = apps.get("autosnap").expect("autosnap is not present");
        assert_eq!(app.command, "bin/autosnap");
    }

    #[test]
    fn test_find_apps_multiple_apps() {
        let tmp_dir = TempDir::new("autosnap").expect("unable to create temporary dir");
        fs::create_dir_all(tmp_dir.path().join("src").join("bin"))
            .expect("unable to create src/bin");

        fs::write(
            tmp_dir.path().join("src").join("bin").join("autosnap.rs"),
            "fn main() {}",
        )
        .expect("unable to write src/bin/autosnap.rs");
        fs::write(
            tmp_dir
                .path()
                .join("src")
                .join("bin")
                .join("autosnap-util.rs"),
            "fn main() {}",
        )
        .expect("unable to write src/bin/autosnap-util.rs");

        let apps = find_apps(tmp_dir, "autosnap").expect("unable to find apps");
        assert!(apps.contains_key("autosnap"));
        assert!(apps.contains_key("autosnap-util"));

        let autosnap = apps.get("autosnap").expect("autosnap is not present");
        assert_eq!(autosnap.command, "bin/autosnap");

        let autosnap_util = apps
            .get("autosnap-util")
            .expect("autosnap-util is not present");
        assert_eq!(autosnap_util.command, "bin/autosnap-util");
    }

    #[test]
    fn test_find_build_packages() {
        let dependency = Dependency {
            name: Name::from_str("openssl-sys").unwrap(),
            version: Version::new(1, 0, 0),
            source: None,
        };

        let package = Package {
            name: Name::from_str("test-package").unwrap(),
            version: Version::new(1, 0, 0),
            source: None,
            checksum: None,
            dependencies: vec![dependency],
            replace: None,
        };

        let lockfile = Lockfile {
            version: Default::default(),
            packages: vec![package],
            root: None,
            metadata: Default::default(),
            patch: Default::default(),
        };

        let build_packages = find_build_packages(&lockfile);
        assert_eq!(build_packages, vec!["libc6-dev", "libssl-dev"]);
    }
}
