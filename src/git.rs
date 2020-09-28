use ansi_term::Colour;
use git2::{ErrorCode::UnbornBranch, Repository, RepositoryState, Status};

use std::fmt::Display;
use std::path::Path;

//borrow from https://github.com/starship/starship/blob/master/src/modules/git_status.rs
#[derive(Default, Debug, Copy, Clone)]
struct RepoStatus {
    pub conflicted: usize,
    pub deleted: usize,
    pub renamed: usize,
    pub modified: usize,
    pub staged: usize,
    pub untracked: usize,
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

    fn is_clean(&self) -> bool {
        self.conflicted == 0
            && self.deleted == 0
            && self.renamed == 0
            && self.modified == 0
            && self.staged == 0
            && self.untracked == 0
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
    repo: Repository,
}

impl GitRepo {
    pub fn new(path: &Path) -> Option<Self> {
        if let Ok(repo) = Repository::discover(path) {
            Some(Self { repo })
        } else {
            None
        }
    }

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

    fn state(&self) -> RepositoryState {
        return self.repo.state();
    }

    fn get_ahead_behind(&self) -> Result<(usize, usize), git2::Error> {
        let branch_name = self.branch().ok_or(git2::Error::from_str("no branch"))?;
        let branch_object = self.repo.revparse_single(&branch_name)?;
        let tracking_branch_name = format!("{}@{{upstream}}", branch_name);
        let tracking_object = self.repo.revparse_single(&tracking_branch_name)?;

        let branch_oid = branch_object.id();
        let tracking_oid = tracking_object.id();

        self.repo.graph_ahead_behind(branch_oid, tracking_oid)
    }

    fn status(&self) -> Result<RepoStatus, git2::Error> {
        let mut status_options = git2::StatusOptions::new();

        let mut repo_status = RepoStatus::default();

        status_options
            .include_untracked(true)
            .renames_from_rewrites(false)
            .renames_head_to_index(false)
            .include_unmodified(false);

        let statuses = self.repo.statuses(Some(&mut status_options))?;

        statuses
            .iter()
            .map(|s| s.status())
            .for_each(|status| repo_status.add(status));

        Ok(repo_status)
    }

    fn branch_string(&self) -> impl Display {
        Colour::Cyan
            .bold()
            .paint(self.branch().or(Some("unknown".to_owned())).unwrap())
    }

    fn ahead_behind_string(&self) -> Box<dyn Display> {
        let ahead_behind = self.get_ahead_behind();
        if ahead_behind.is_err() {
            return Box::new("");
        }
        let (ahead, behind) = ahead_behind.unwrap();

        if ahead == 0 && behind == 0 {
            return Box::new(" ");
        }

        let get_mark = |count: usize, mark: &'static str| -> String {
            if count > 0 {
                format!("{}{}", Colour::Yellow.paint(mark), count)
            } else {
                String::default()
            }
        };

        Box::new(format!(
            " - {}{}",
            get_mark(ahead, "↑"),
            get_mark(behind, "↓")
        ))
    }

    fn steate_string(&self) -> String {
        let state_str = match self.state() {
            RepositoryState::Clean => "",
            RepositoryState::Merge => "merge",
            RepositoryState::Revert => "revert",
            RepositoryState::RevertSequence => "revert-sequence",
            RepositoryState::CherryPick => "cherry-pick",
            RepositoryState::CherryPickSequence => "cherry-pick sequence",
            RepositoryState::Bisect => "bitsect",
            RepositoryState::Rebase => "rebase",
            RepositoryState::RebaseInteractive => "rebase-i",
            RepositoryState::RebaseMerge => "rebase-merge",
            RepositoryState::ApplyMailbox => "apply-mailbox",
            RepositoryState::ApplyMailboxOrRebase => "apply-mailbox-rebase",
        };

        if state_str.is_empty() {
            "".to_owned()
        } else {
            format!("|{}|", Colour::Blue.paint(state_str))
        }
    }

    fn status_string(&self) -> Box<dyn Display> {
        let status = self.status();
        if status.is_err() {
            return Box::new(Colour::Red.paint("unknown"));
        }
        let status = status.unwrap();
        if status.is_clean() {
            return Box::new(Colour::Green.paint(""));
        }

        let get_ico = |count: usize, mark: &'static str| -> &'static str {
            if count > 0 {
                mark
            } else {
                ""
            }
        };

        Box::new(format!(
            "{}{}{}{}",
            Colour::Green.paint(get_ico(status.staged, "s")),
            Colour::Yellow.paint(get_ico(status.modified, "m")),
            Colour::Blue.paint(get_ico(status.untracked, "u")),
            Colour::Red.paint(get_ico(status.conflicted, "c"))
        ))
    }

    fn get_stash_count(&mut self) -> usize {
        let mut count = 0;
        self.repo
            .stash_foreach(|_, _, _| {
                count += 1;
                true
            })
            .unwrap_or_default();

        count
    }

    fn stash_count_string(&mut self) -> String {
        match self.get_stash_count() {
            0 => "".to_owned(),
            count => format!(" {}({})", Colour::Blue.paint("S"), count),
        }
    }

    pub fn print(&mut self) {
        print!(
            "on {}({}){}{}{}",
            self.branch_string(),
            Colour::Blue.paint(self.status_string().to_string()),
            self.steate_string(),
            self.ahead_behind_string(),
            self.stash_count_string()
        )
    }
}
