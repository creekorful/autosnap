use std::collections::BTreeMap;
use std::error::Error;
use std::process::exit;
use std::{env, fs};

use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SnapFile {
    name: String,
    base: String,
    version: String,
    summary: String,
    description: String,
    license: String,
    grade: String,
    confinement: String,
    parts: BTreeMap<String, SnapPart>,
    apps: BTreeMap<String, SnapApp>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SnapPart {
    plugin: String,
    source: String,
    #[serde(rename = "build-packages")]
    build_packages: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SnapApp {
    command: String,
    plugs: Vec<String>,
}

fn main() {
    let matches = App::new("autosnap")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Automatically make Snap package from github repository")
        .arg(
            Arg::with_name("src")
                .value_name("SRC")
                .required(true)
                .help("The github repository (example: github.com/creekorful/osync)."),
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    let mut src = matches.value_of("src").unwrap().to_string();
    // preprend https:// if not present
    if !src.starts_with("http") {
        src = format!("https://{}", src);
    }

    let src = match Url::parse(&src) {
        Ok(src) => src,
        Err(_) => {
            eprintln!("Invalid repository url {}", src);
            exit(1);
        }
    };

    println!("Starting packaging of {}", src);

    if let Err(e) = package(&src) {
        eprintln!("Error encountered while packaging: {}", e);
        exit(1);
    }
}

fn package(repo_uri: &Url) -> Result<SnapFile, Box<dyn Error>> {
    let cwd = env::current_dir()?;
    let repo_name = repo_uri.path_segments().unwrap().last().unwrap();
    let path = cwd.join(repo_name);

    // Clone the repository
    git2::Repository::clone(repo_uri.as_str(), &path)?;

    // Determinate if not already packaged
    if path.join("snapcraft.yaml").exists() || path.join("snap").join("snapcraft.yaml").exists() {
        fs::remove_dir_all(path)?;
        return Err(format!("{} is already packaged", repo_uri).into());
    }

    // TODO Identify the project license
    // using https://github.com/jpeddicord/askalono

    let snap = SnapFile {
        name: repo_name.to_string(),
        base: "core18".to_string(),
        version: "git".to_string(),
        summary: "".to_string(),     // Extract from Github API
        description: "".to_string(), // Extract from README.md
        license: "".to_string(),     // Extract using askalono
        grade: "stable".to_string(),
        confinement: "strict".to_string(),
        parts: Default::default(),
        apps: Default::default(),
    };

    Err("not implemented".into())
}
