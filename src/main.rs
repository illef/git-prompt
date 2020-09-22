use git2::Repository;
use std::path::{Path, PathBuf};

pub struct GitRepo {
    /// The current working directory that starship is being called in.
    current_dir: PathBuf,
    repo: Repository,
}

impl GitRepo {
    pub fn new(path: &Path) -> Option<Self> {
        if let Ok(repo) = Repository::discover(path) {
            Some(Self {
                current_dir: path.to_owned(),
                repo,
            })
        } else {
            None
        }
    }
}

fn main() {
    println!("Hello, world!");
}
