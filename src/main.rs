mod git;

use git::*;

fn main() {
    let repo = GitRepo::new(&std::env::current_dir().unwrap());
    repo.and_then::<(), _>(|mut repo| {
        repo.print();
        None
    });
}
