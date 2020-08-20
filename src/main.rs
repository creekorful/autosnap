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
    print!("{}", src);
}
