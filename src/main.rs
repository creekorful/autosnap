use std::error::Error;
use std::process::exit;
use std::{env, fs};

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

fn package(repo_uri: &Url) -> Result<(), Box<dyn Error>> {
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

    Err("not implemented".into())
}
