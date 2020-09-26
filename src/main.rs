mod git;

use git::*;

fn main() {
    let repo = GitRepo::new(&std::env::current_dir().unwrap());
    if repo.is_none() {
        std::process::exit(0);
    }
    let repo = repo.unwrap();

    repo.print();
}
