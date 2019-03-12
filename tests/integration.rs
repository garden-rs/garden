extern crate subprocess;
extern crate garden;

#[cfg(test)]
mod integration {

use super::garden::cmd;

/// Cleanup and create a bare repository for cloning
fn setup(name: &str, path: &str) {
    let cmd = ["./setup.sh", name];
    let exec = garden::cmd::exec_in_dir(&cmd, path);
    let exit_status = garden::cmd::status(exec.join());
    assert_eq!(exit_status, 0);
}

fn teardown(path: &str) {
    if let Err(err) = std::fs::remove_dir_all(path) {
        assert!(false, format!("unable to remove '{}': {}", path, err));
    }
}

/// `garden init` clones repositories
#[test]
fn garden_init_clone() {
    setup("clone", "tests/init");

    // garden init examples/tree
    let cmd = [
        "./target/debug/garden",
        "--chdir", "./tests/init/clone",
        "--config", "../garden.yaml",
        "init", "example/tree",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let exit_status = garden::cmd::status(exec.join());
    assert_eq!(exit_status, 0);

    // Ensure the repository was created
    let mut repo = std::path::PathBuf::from("tests");
    assert!(repo.exists());
    // tests/init
    repo.push("init");
    assert!(repo.exists());
    // tests/init/clone/example
    repo.push("clone");
    assert!(repo.exists());
    // tests/init/clone/example
    repo.push("example");
    assert!(repo.exists());
    // tests/init/clone/example/tree
    repo.push("tree");
    assert!(repo.exists());
    // tests/init/clone/example/tree/repo
    repo.push("repo");
    assert!(repo.exists());
    // tests/init/clone/example/tree/repo/.git
    repo.push(".git");
    assert!(repo.exists());

    teardown("tests/init/clone");
}


/// `garden init` sets up remotes
#[test]
fn garden_init_remotes() {
    setup("remotes", "tests/init");

    // garden init examples/tree
    let cmd = [
        "./target/debug/garden",
        "--chdir", "./tests/init/remotes",
        "--config", "../garden.yaml",
        "init", "example/tree",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let exit_status = garden::cmd::status(exec.join());
    assert_eq!(exit_status, 0);
    // remote.origin.url is a read-only git:// URL
    {
        let command = ["git", "config", "remote.origin.url"];
        let exec = cmd::exec_in_dir(
            &command, "tests/init/remotes/example/tree/repo");
        if let Ok(x) = cmd::capture_stdout(exec) {
            let output = cmd::trim_stdout(&x);
            assert!(output.ends_with("/tests/init/remotes/repos/example.git"),
            format!("{} does not end with /tests/init/clone/repos/example.git",
                    output));
        } else {
            assert!(false, "'git config remote.origin.url' had an error");
        }
    }

    // remote.publish.url is a ssh push URL
    {
        let command = ["git", "config", "remote.publish.url"];
        let exec = cmd::exec_in_dir(&command, "tests/init/remotes/example/tree/repo");
        if let Ok(x) = cmd::capture_stdout(exec) {
            let output = cmd::trim_stdout(&x);
            assert_eq!(output, "git@github.com:user/example.git");
        } else {
            assert!(false, "'git config remote.publish.url' had an error");
        }
    }

    teardown("tests/init/remotes");
}

}  // integration
