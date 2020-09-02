use crate::generator::{Generator, Provider};
use crate::snap::{App, Part};
use crate::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct PythonGenerator {
    setup_py: SetupPy,
    source_name: String,
}

pub struct PythonProvider {}

#[derive(Default)]
pub struct SetupPy {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
}

impl Provider<PythonGenerator> for PythonProvider {
    fn provide<P: AsRef<Path>>(source_path: P, source_name: &str) -> Result<PythonGenerator> {
        // Make sure python is on path
        Command::new("python")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // Set fields
        Ok(PythonGenerator {
            setup_py: SetupPy {
                name: execute_cmd(&source_path, "name"),
                version: execute_cmd(&source_path, "version"),
                description: execute_cmd(&source_path, "description"),
            },
            source_name: source_name.to_string(),
        })
    }

    fn can_provide<P: AsRef<Path>>(source_path: P) -> bool {
        source_path.as_ref().join("setup.py").exists()
    }
}

impl Generator for PythonGenerator {
    fn name(&self) -> Result<Option<String>> {
        Ok(self.setup_py.name.clone())
    }

    fn version(&self) -> Result<Option<String>> {
        Ok(self.setup_py.version.clone())
    }

    fn summary(&self) -> Result<Option<String>> {
        Ok(self.setup_py.description.clone())
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
                plugin: "python".to_string(),
                source: ".".to_string(),
                build_packages: None,
                stage_packages: None,
                go_import_path: None,
                python_version: Some("python3".to_string()), // TODO
            },
        );

        Ok(parts)
    }

    fn apps(&self) -> Result<BTreeMap<String, App>> {
        let mut apps = BTreeMap::default();
        apps.insert(
            self.source_name.clone(),
            App {
                command: "TODO".to_string(),
                plugs: None,
            },
        );
        Ok(apps)
    }
}

fn execute_cmd<P: AsRef<Path>>(source_path: P, field: &str) -> Option<String> {
    let cmd = Command::new("python")
        .current_dir(&source_path)
        .arg("setup.py")
        .arg(format!("--{}", field))
        .output();

    if cmd.is_err() {
        return None;
    }

    let cmd = cmd.unwrap();
    String::from_utf8(cmd.stdout)
        .ok()
        .map(|s| s.replace('\n', ""))
}
