use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};
use url::Url;

mod snap;

pub fn clone_repo(repo_uri: &Url) -> Result<PathBuf, Box<dyn Error>> {
    let cwd = env::current_dir()?;
    let repo_name = repo_uri.path_segments().unwrap().last().unwrap();
    let path = cwd.join(repo_name);

    // Clone the repository
    git2::Repository::clone(repo_uri.as_str(), &path)?;

    Ok(path)
}

pub fn package_repo<P: AsRef<Path>>(repo_path: P) -> Result<snap::File, Box<dyn Error>> {
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

    let snap = snap::File::new(repo_name, "", "", "");

    Err("not implemented".into())
}
