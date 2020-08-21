use std::process::exit;

use autosnap::{clone_repo, package_repo};
use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use url::Url;

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

    let snap = match package_repo(&path) {
        Ok(snap) => snap,
        Err(e) => {
            eprintln!("Error encountered while packaging repository: {}", e);
            exit(1);
        }
    };
}
