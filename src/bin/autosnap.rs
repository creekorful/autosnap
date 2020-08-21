use std::process::exit;

use autosnap::snap::SNAPCRAFT_YAML;
use autosnap::{clone_repo, package_repo};
use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use std::fs;
use url::Url;

fn main() {
    let matches = App::new("autosnap")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Automatically make Snap package from github repository")
        .arg(
            Arg::with_name("repo")
                .value_name("REPO")
                .required(true)
                .help("The github repository (example: https://github.com/creekorful/osync)."),
        )
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();

    let src = matches.value_of("repo").unwrap().to_string();
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
            fs::remove_dir_all(&path).expect("unable to delete cloned repository");
            eprintln!("Error encountered while packaging repository: {}", e);
            exit(1);
        }
    };

    // serialize file into yaml
    let yaml = match serde_yaml::to_string(&snap) {
        Ok(yaml) => yaml,
        Err(e) => {
            eprintln!("Error encountered while serializing snap file: {}", e);
            exit(1);
        }
    };

    // write snap file inside the cloned repository
    if let Err(e) = fs::write(path.join(SNAPCRAFT_YAML), yaml) {
        eprintln!("Error encountered while writing snapcraft.yaml: {}", e);
        exit(1);
    }

    println!("Successfully packaged {}!", snap.name);
    println!(
        "The snapcraft file is stored at {}",
        path.join(SNAPCRAFT_YAML).display()
    );
    println!(
        "Please fix any TODO in the file and run `cd {} && snapcraft`",
        path.display()
    );
}
