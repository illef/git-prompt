mod git;

use git::*;

fn main() {
    let repo = GitRepo::new(&std::env::current_dir().unwrap());
    if repo.is_none() {
        return;
    }
    let mut repo = repo.unwrap();
    if std::env::args().filter(|arg| arg == "--json").next() == None {
        repo.print();
    } else {
        print!("{}", serde_json::to_string(&repo.into_info()).unwrap());
    }
}
