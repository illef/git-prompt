use git2::{ErrorCode::UnbornBranch, Repository, RepositoryState};

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

    //borrow from https://github.com/starship/starship/blob/a245d54cdbbcf5480571bce1a055299b815c6109/src/context.rs#L349
    fn branch(&self) -> Option<String> {
        let head = match self.repo.head() {
            Ok(reference) => reference,
            Err(e) => {
                return if e.code() == UnbornBranch {
                    // HEAD should only be an unborn branch if the repository is fresh,
                    // in that case read directly from `.git/HEAD`
                    let mut head_path = self.repo.path().to_path_buf();
                    head_path.push("HEAD");

                    // get first line, then last path segment
                    std::fs::read_to_string(&head_path)
                        .ok()?
                        .lines()
                        .next()?
                        .trim()
                        .split('/')
                        .last()
                        .map(|r| r.to_owned())
                } else {
                    None
                };
            }
        };

        let shorthand = head.shorthand();

        shorthand.map(std::string::ToString::to_string)
    }
}

fn main() {
    println!(
        "{}",
        GitRepo::new(&std::env::current_dir().unwrap())
            .unwrap()
            .branch()
            .unwrap()
    );
}
