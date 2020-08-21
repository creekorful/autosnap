#[macro_use]
extern crate log;
extern crate simple_logger;

use std::process::exit;

use autosnap::snap::SNAPCRAFT_YAML;
use autosnap::{clone_repo, package_repo};
use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use log::Level;
use std::fs;
use std::str::FromStr;
use url::Url;

fn main() {
    let matches = App::new("autosnap")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Automatically make Snap package from git repository")
        .arg(
            Arg::with_name("repo")
                .value_name("REPO")
                .required(true)
                .help("The git repository (example: https://github.com/creekorful/osync)."),
        )
        .arg(
            Arg::with_name("log-level")
                .long("log-level")
                .default_value("info"),
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    // configure logging
    let log_level = matches.value_of("log-level").unwrap();
    let log_level = match Level::from_str(log_level) {
        Ok(level) => level,
        Err(e) => {
            eprintln!("Error while configuring logging: {}", e);
            exit(1);
        }
    };
    simple_logger::init_with_level(log_level).unwrap();

    let src = matches.value_of("repo").unwrap().to_string();
    let src = match Url::parse(&src) {
        Ok(src) => src,
        Err(_) => {
            error!("Invalid repository url {}", src);
            exit(1);
        }
    };

    info!("Starting packaging of {}", src);

    // first of all clone the remote repository
    let path = match clone_repo(&src) {
        Ok(path) => path,
        Err(e) => {
            error!("Error encountered while cloning repository: {}", e);
            exit(1);
        }
    };

    let snap = match package_repo(&path) {
        Ok(snap) => snap,
        Err(e) => {
            fs::remove_dir_all(&path).expect("unable to delete cloned repository");
            error!("Error encountered while packaging repository: {}", e);
            exit(1);
        }
    };

    // serialize file into yaml
    let yaml = match serde_yaml::to_string(&snap) {
        Ok(yaml) => yaml,
        Err(e) => {
            error!("Error encountered while serializing snap file: {}", e);
            exit(1);
        }
    };

    // write snap file inside the cloned repository
    if let Err(e) = fs::write(path.join(SNAPCRAFT_YAML), yaml) {
        error!("Error encountered while writing {}: {}", SNAPCRAFT_YAML, e);
        exit(1);
    }

    info!("Successfully packaged {}!", snap.name);
    info!(
        "The snapcraft file is stored at {}",
        path.join(SNAPCRAFT_YAML).display()
    );
    info!(
        "Please fix any TODO in the file and run `cd {} && snapcraft`",
        path.display()
    );
}
