use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SNAPCRAFT_YAML: &str = "snapcraft.yaml";

/// This structure represent a Snap (snapcraft.yaml) file.
/// See this [link](https://snapcraft.io/docs/snapcraft-yaml-reference) for more information.
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
    pub fn new(name: &str) -> File {
        File {
            name: name.to_string(),
            base: "core18".to_string(),
            version: "TODO".to_string(),
            summary: "TODO".to_string(),
            description: "TODO".to_string(),
            license: "TODO".to_string(),
            grade: "devel".to_string(),
            confinement: "devmode".to_string(), // TODO switch to strict when we manage plugs
            parts: Default::default(),
            apps: Default::default(),
        }
    }
}

/// This structure represent a Part.
/// See this [link](https://snapcraft.io/docs/adding-parts) for more information.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Part {
    pub plugin: String,
    pub source: String,
    #[serde(rename = "build-packages")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_packages: Option<Vec<String>>,
    #[serde(rename = "stage-packages")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_packages: Option<Vec<String>>,
    #[serde(rename = "go-importpath")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub go_import_path: Option<String>,
    #[serde(rename = "python-version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub python_version: Option<String>,
}

/// This structure represent an Application.
/// See this [link](https://snapcraft.io/docs/commands-and-aliases) for more information.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct App {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugs: Option<Vec<String>>,
}
