#[macro_use]
extern crate log;
extern crate simple_logger;

use std::process::exit;

use autosnap::snap::SNAPCRAFT_YAML;
use autosnap::{fetch_source, package_source};
use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use log::Level;
use std::fs;
use std::str::FromStr;
use url::Url;

fn main() {
    let matches = App::new("autosnap")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Automatically make Snap package from source code")
        .arg(
            Arg::with_name("source")
                .value_name("SRC")
                .required(true)
                .help("The source location (example: https://github.com/creekorful/osync)."),
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

    let src = matches.value_of("source").unwrap().to_string();

    info!("Starting packaging of {}", src);

    // first of all, if its a remote source, fetch it
    let path = match Url::parse(&src) {
        Ok(src) => match fetch_source(&src) {
            Ok(path) => path,
            Err(e) => {
                error!("Error while fetching source: {}", e);
                exit(1);
            }
        },
        Err(_) => src.into(),
    };

    // package the source code
    let snap = match package_source(&path) {
        Ok(snap) => snap,
        Err(e) => {
            error!("Error encountered while packaging source: {}", e);
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

    // write snap file inside the source root
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
