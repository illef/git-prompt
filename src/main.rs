use git2::{ErrorCode::UnbornBranch, Repository, RepositoryState, Status};

use std::path::{Path, PathBuf};

#[derive(Default, Debug, Copy, Clone)]
struct RepoStatus {
    conflicted: usize,
    deleted: usize,
    renamed: usize,
    modified: usize,
    staged: usize,
    untracked: usize,
}

impl RepoStatus {
    fn is_conflicted(status: Status) -> bool {
        status.is_conflicted()
    }

    fn is_deleted(status: Status) -> bool {
        status.is_wt_deleted() || status.is_index_deleted()
    }

    fn is_renamed(status: Status) -> bool {
        status.is_wt_renamed() || status.is_index_renamed()
    }

    fn is_modified(status: Status) -> bool {
        status.is_wt_modified()
    }

    fn is_staged(status: Status) -> bool {
        status.is_index_modified() || status.is_index_new()
    }

    fn is_untracked(status: Status) -> bool {
        status.is_wt_new()
    }

    fn add(&mut self, s: Status) {
        self.conflicted += RepoStatus::is_conflicted(s) as usize;
        self.deleted += RepoStatus::is_deleted(s) as usize;
        self.renamed += RepoStatus::is_renamed(s) as usize;
        self.modified += RepoStatus::is_modified(s) as usize;
        self.staged += RepoStatus::is_staged(s) as usize;
        self.untracked += RepoStatus::is_untracked(s) as usize;
    }
}

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

    fn status(&self) -> Result<RepoStatus, git2::Error> {
        let mut status_options = git2::StatusOptions::new();

        let mut repo_status = RepoStatus::default();

        status_options
            .include_untracked(true)
            .renames_from_rewrites(true)
            .renames_head_to_index(true)
            .include_unmodified(true);

        let statuses = self.repo.statuses(Some(&mut status_options))?;

        if statuses.is_empty() {
            return Err(git2::Error::from_str("Repo has no status"));
        }

        statuses
            .iter()
            .map(|s| s.status())
            .for_each(|status| repo_status.add(status));

        Ok(repo_status)
    }
}

fn main() {
    let repo = GitRepo::new(&std::env::current_dir().unwrap()).unwrap();

    println!("{} {:?}", repo.branch().unwrap(), repo.status().unwrap());
}
