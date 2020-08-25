use std::{fs, io};
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::generator::Generator;
use crate::snap::{App, File, Part};

// the go.mod file
#[derive(Default)]
struct ModFile {
    pub go_version: String,
    pub import_path: String,
    // TODO dependencies etc
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
        let mod_file = parse_mod(source_path.as_ref().join("go.mod"))?;
        debug!(
            "setting go-import-path to {} (using go.mod)",
            mod_file.import_path
        );

        // generate parts
        let mut parts: BTreeMap<String, Part> = BTreeMap::new();
        parts.insert(
            snap.name.clone(),
            Part {
                plugin: "go".to_string(),
                source: ".".to_string(),
                build_packages: Some(vec!["gcc".to_string(), "libc6-dev".to_string()]),
                stage_packages: None,
                go_import_path: Some(mod_file.import_path),
            },
        );
        snap.parts = parts;

        // find executables built by the go module
        let executables = find_executables(source_path)?;
        if executables.is_empty() {
            return Err("no executable(s) found".into());
        }

        let mut apps: BTreeMap<String, App> = BTreeMap::new();
        for executable in executables {
            apps.insert(
                executable.to_string(),
                App {
                    command: format!("bin/{}", executable).to_string(),
                    plugs: None,
                },
            );
        }
        snap.apps = apps;

        Ok(snap)
    }
}

fn find_executables<P: AsRef<Path>>(path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let mut executables: Vec<String> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;

        // recursive search
        if entry.file_type()?.is_dir() {
            return find_executables(entry.path());
        }

        if let Some(filename) = entry.file_name().to_str() {
            // TODO better
            if filename.ends_with(".go") {
                let content = fs::read_to_string(entry.path())?;
                if content.contains("package main") && content.contains("func main()") {
                    let executable_name = filename.replace(".go", "");
                    debug!("found executable (name: {})", executable_name);
                    executables.push(executable_name);
                }
            }
        }
    }

    Ok(executables)
}

fn parse_mod<P: AsRef<Path>>(path: P) -> Result<ModFile, Box<dyn Error>> {
    let mut mod_file = ModFile::default();

    for line in read_lines(&path)? {
        if line.starts_with("module") {
            mod_file.import_path = line.replace("module ", "");
        }
        if line.starts_with("go") {
            mod_file.go_version = line.replace("go ", "");
        }
    }

    Ok(mod_file)
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<Vec<String>> {
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().filter_map(io::Result::ok).collect())
}
