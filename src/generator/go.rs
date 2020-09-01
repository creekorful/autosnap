use crate::generator::{Generator, Provider};
use crate::snap::{App, Part};
use crate::Result;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::{fs, io};
use walkdir::WalkDir;

pub struct GoGenerator {
    mod_file: ModFile,
    source_path: PathBuf,
    source_name: String,
}

pub struct GoProvider {}

// the go.mod file
#[derive(Default)]
struct ModFile {
    pub go_version: String,
    pub import_path: String,
}

impl ModFile {
    fn load<P: AsRef<Path>>(path: P) -> Result<ModFile> {
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
}

impl Provider<GoGenerator> for GoProvider {
    fn provide<P: AsRef<Path>>(source_path: P, source_name: &str) -> Result<GoGenerator> {
        let mod_file = ModFile::load(source_path.as_ref().join("go.mod"))?;
        Ok(GoGenerator {
            mod_file,
            source_path: source_path.as_ref().to_path_buf(),
            source_name: source_name.to_string(),
        })
    }

    fn can_provide<P: AsRef<Path>>(source_path: P) -> bool {
        source_path.as_ref().join("go.mod").exists()
    }
}

impl Generator for GoGenerator {
    fn name(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn version(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn summary(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn description(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn license(&self) -> Result<Option<String>> {
        Ok(None)
    }

    fn parts(&self) -> Result<BTreeMap<String, Part>> {
        let mut parts = BTreeMap::default();
        parts.insert(
            self.source_name.clone(),
            Part {
                plugin: "go".to_string(),
                source: ".".to_string(),
                build_packages: Some(vec!["gcc".to_string(), "libc6-dev".to_string()]),
                stage_packages: None,
                go_import_path: Some(self.mod_file.import_path.clone()),
            },
        );

        Ok(parts)
    }

    fn apps(&self) -> Result<BTreeMap<String, App>> {
        let mut apps = BTreeMap::default();
        let executables = find_executables(self.source_path.clone())?;
        for executable in executables {
            apps.insert(
                executable.to_string(),
                App {
                    command: format!("bin/{}", executable).to_string(),
                    plugs: None,
                },
            );
        }

        Ok(apps)
    }
}

fn find_executables<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let mut executables: Vec<String> = Vec::new();

    for entry in WalkDir::new(&path) {
        let entry = entry?;

        if let Some(filename) = entry.file_name().to_str() {
            if filename.ends_with(".go") {
                let content = fs::read_to_string(entry.path())?;
                if content.contains("package main") && content.contains("func main()") {
                    let executable_name = filename.replace(".go", "");
                    log::debug!("Found executable (name: {})", executable_name);
                    executables.push(executable_name);
                }
            }
        }
    }

    Ok(executables)
}

fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<Vec<String>> {
    let file = fs::File::open(filename)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().filter_map(io::Result::ok).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_parse_mod() {
        let tmp_dir = TempDir::new("autosnap").expect("unable to create temporary dir");
        let mod_file = "module github.com/creekorful/trandoshan\ngo 1.14";
        fs::write(tmp_dir.path().join("go.mod"), mod_file).expect("unable to write go.mod");

        let mod_file = ModFile::load(tmp_dir.path().join("go.mod"));
        assert!(mod_file.is_ok());

        let mod_file = mod_file.unwrap();
        assert_eq!(mod_file.import_path, "github.com/creekorful/trandoshan");
        assert_eq!(mod_file.go_version, "1.14");
    }

    #[test]
    fn test_read_lines() {
        let tmp_dir = TempDir::new("autosnap").expect("unable to create temporary dir");
        let mod_file = "module github.com/creekorful/trandoshan\ngo 1.14";
        fs::write(tmp_dir.path().join("go.mod"), mod_file).expect("unable to write go.mod");

        let lines = read_lines(tmp_dir.path().join("go.mod")).expect("unable to read go.mod");
        assert_eq!(
            lines,
            vec!["module github.com/creekorful/trandoshan", "go 1.14"]
        );
    }

    #[test]
    fn test_find_executables() {
        let tmp_dir = TempDir::new("autosnap").expect("unable to create temporary dir");
        fs::create_dir_all(tmp_dir.path().join("cmd").join("foo"))
            .expect("unable to create cmd/foo");
        fs::create_dir_all(tmp_dir.path().join("cmd").join("bar"))
            .expect("unable to create cmd/bar");
        fs::create_dir_all(tmp_dir.path().join("cmd").join("baz"))
            .expect("unable to create cmd/baz");

        fs::write(
            tmp_dir.path().join("cmd").join("foo").join("foo.go"),
            "package main\nfunc main(){}",
        )
        .expect("unable to write cmd/foo/foo.go");
        fs::write(
            tmp_dir.path().join("cmd").join("bar").join("bar.go"),
            "package main\nfunc main(){}",
        )
        .expect("unable to write cmd/bar/bar.go");
        fs::write(
            tmp_dir.path().join("cmd").join("baz").join("baz.go"),
            "package main\nfunc main(){}",
        )
        .expect("unable to write cmd/baz/baz.go");

        let executables = find_executables(tmp_dir);
        assert!(executables.is_ok());

        let mut executables = executables.unwrap();
        executables.sort();
        assert_eq!(executables, vec!["bar", "baz", "foo"]);
    }
}
