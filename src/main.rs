use std::error::Error;
use std::process::exit;

use clap::{App, AppSettings, Arg, crate_authors, crate_version};

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

    let src = matches.value_of("src").unwrap();
    println!("Starting packaging of {}", src);

    if let Err(e) = package(src) {
        eprintln!("Error encountered while packaging: {}", e);
        exit(1);
    }
}

fn package(repo_uri: &str) -> Result<(), Box<dyn Error>> {
    Err("not implemented".into())
}