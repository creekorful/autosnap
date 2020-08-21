use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SNAPCRAFT_YAML: &str = "snapcraft.yaml";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct File {
    pub name: String,
    pub base: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub license: String,
    pub grade: String,
    pub confinement: String,
    pub parts: BTreeMap<String, Part>,
    pub apps: BTreeMap<String, App>,
}

impl File {
    pub fn new(name: &str, summary: &str, description: &str, license: &str) -> File {
        File {
            name: name.to_string(),
            base: "core18".to_string(),
            version: "git".to_string(),
            summary: summary.to_string(),
            description: description.to_string(),
            license: license.to_string(),
            grade: "stable".to_string(),
            confinement: "classic".to_string(), // TODO switch to strict when we manage plugs
            parts: Default::default(),
            apps: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Part {
    pub plugin: String,
    pub source: String,
    #[serde(rename = "build-packages")]
    pub build_packages: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct App {
    pub command: String,
    pub plugs: Vec<String>,
}
