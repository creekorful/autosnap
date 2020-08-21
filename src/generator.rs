use crate::snap::File;
use std::error::Error;

/// A Generator is a autosnap extension that know how to package
/// a specific language.
pub trait Generator {
    fn generate(&self, snap: &File) -> Result<File, Box<dyn Error>>;
}

/// Generators contains the list of supported snapcraft generator
pub enum Generators {
    Rust(RustGenerator),
}

impl Generator for Generators {
    fn generate(&self, snap: &File) -> Result<File, Box<dyn Error>> {
        match *self {
            Generators::Rust(ref generator) => generator.generate(snap),
        }
    }
}

/// The GeneratorBuilder allow to return specific generator
/// based on project language
pub struct GeneratorBuilder {}

impl GeneratorBuilder {
    pub fn get() -> Generators {
        Generators::Rust(RustGenerator {})
    }
}

/// The RustGenerator provide autosnap capability for Rust project
pub struct RustGenerator {}

impl Generator for RustGenerator {
    fn generate(&self, snap: &File) -> Result<File, Box<dyn Error>> {
        unimplemented!()
    }
}
