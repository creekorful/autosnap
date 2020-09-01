use crate::generator::{Generator, Provider};
use crate::Result;
use std::path::Path;
use crate::snap::{Part, App};
use std::collections::BTreeMap;

pub struct PythonGenerator {}

pub struct PythonProvider {}

impl Provider<PythonGenerator> for PythonProvider {
    fn provide<P: AsRef<Path>>(source_path: P, source_name: &str) -> Result<PythonGenerator> {
        unimplemented!()
    }

    fn can_provide<P: AsRef<Path>>(source_path: P) -> bool {
        source_path.as_ref().join("setup.py").exists()
    }
}

impl Generator for PythonGenerator {
    fn name(&self) -> Result<Option<String>> {
        unimplemented!()
    }

    fn version(&self) -> Result<Option<String>> {
        unimplemented!()
    }

    fn summary(&self) -> Result<Option<String>> {
        unimplemented!()
    }

    fn description(&self) -> Result<Option<String>> {
        unimplemented!()
    }

    fn license(&self) -> Result<Option<String>> {
        unimplemented!()
    }

    fn parts(&self) -> Result<BTreeMap<String, Part>> {
        unimplemented!()
    }

    fn apps(&self) -> Result<BTreeMap<String, App>> {
        unimplemented!()
    }
}
