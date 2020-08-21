use std::collections::BTreeMap;
use std::error::Error;
use std::process::exit;
use std::{env, fs};

use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
                .help("The github repository (example: https://github.com/creekorful/osync)."),
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    let src = matches.value_of("src").unwrap().to_string();
    let src = match Url::parse(&src) {
        Ok(src) => src,
        Err(_) => {
            eprintln!("Invalid repository url {}", src);
            exit(1);
        }
    };

    println!("Starting packaging of {}", src);

    // first of all clone the remote repository
    let path = match clone_repo(&src) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error encountered while cloning repository: {}", e);
            exit(1);
        }
    };

    if let Err(e) = package_repo(&path) {
        eprintln!("Error encountered while packaging: {}", e);
        exit(1);
    }
}

fn clone_repo(repo_uri: &Url) -> Result<PathBuf, Box<dyn Error>> {
    let cwd = env::current_dir()?;
    let repo_name = repo_uri.path_segments().unwrap().last().unwrap();
    let path = cwd.join(repo_name);

    // Clone the repository
    git2::Repository::clone(repo_uri.as_str(), &path)?;

    Ok(path)
}

fn package_repo<P: AsRef<Path>>(repo_path: P) -> Result<SnapFile, Box<dyn Error>> {
    let repo_name = repo_path.as_ref().file_name().unwrap().to_str().unwrap();

    // Determinate if not already packaged
    if repo_path.as_ref().join("snapcraft.yaml").exists()
        || repo_path
            .as_ref()
            .join("snap")
            .join("snapcraft.yaml")
            .exists()
    {
        fs::remove_dir_all(&repo_path)?;
        return Err(format!("{} is already packaged", repo_name).into());
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
