use std::collections::BTreeMap;
use std::path::Path;

use crate::generator::{Generator, Provider};
use crate::snap::{App, Part};
use crate::Result;

const JAVA_MAIN: &str = "public static void main";
const KOTLIN_MAIN: &str = "fun main";

pub struct GradleGenerator {
    source_name: String,
}

pub struct GradleProvider {}

impl Provider<GradleGenerator> for GradleProvider {
    fn provide<P: AsRef<Path>>(source_path: P, source_name: &str) -> Result<GradleGenerator> {
        Ok(GradleGenerator {
            source_name: source_name.to_string(),
        })
    }

    fn can_provide<P: AsRef<Path>>(source_path: P) -> bool {
        source_path.as_ref().join("settings.gradle").exists()
    }
}

impl Generator for GradleGenerator {
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
                plugin: "gradle".to_string(),
                source: ".".to_string(),
                build_packages: None,
                stage_packages: None,
                go_import_path: None,
                python_version: None,
            },
        );

        Ok(parts)
    }

    fn apps(&self) -> Result<BTreeMap<String, App>> {
        let apps = BTreeMap::default();
        Ok(apps)
    }
}
