use crate::generator::{Generator, Provider};
use crate::snap::{App, Part};
use crate::Result;
use std::collections::BTreeMap;
use std::path::Path;

pub struct PythonGenerator {
    setup_py: SetupPy,
}

pub struct PythonProvider {}

pub struct SetupPy {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    python_requires: Option<String>,
}

impl Provider<PythonGenerator> for PythonProvider {
    fn provide<P: AsRef<Path>>(source_path: P, source_name: &str) -> Result<PythonGenerator> {
        // TODO parse setup.py file
        unimplemented!()
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
        unimplemented!()
    }

    fn apps(&self) -> Result<BTreeMap<String, App>> {
        unimplemented!()
    }
}
