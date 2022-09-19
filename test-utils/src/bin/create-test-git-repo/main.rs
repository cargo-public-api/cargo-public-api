use test_utils::create_test_git_repo;

fn main() {
    let mut args = std::env::args_os();
    create_test_git_repo(args.nth(1).unwrap(), args.next().unwrap());
}
