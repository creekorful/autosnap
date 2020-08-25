use crate::generator::Generator;
use crate::snap::{File, Part};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

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
