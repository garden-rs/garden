extern crate subprocess;
extern crate garden;

#[cfg(test)]
mod integration {

use super::garden::cmd;

#[test]
fn garden_init() {
    // Cleanup and create the bare repository used by this test
    {
        let cmd = ["./setup.sh"];
        let exec = garden::cmd::exec_in_dir(&cmd, "tests/init");
        let exit_status = garden::cmd::status(exec.join());
        assert_eq!(exit_status, 0);
    }

    // garden init examples/tree
    let cmd = [
        "./target/debug/garden",
        "--chdir", "./tests/init",
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
    // tests/init/example
    repo.push("example");
    assert!(repo.exists());
    // tests/init/example/tree
    repo.push("tree");
    assert!(repo.exists());
    // tests/init/example/tree/repo
    repo.push("repo");
    assert!(repo.exists());
    // tests/init/example/tree/repo/.git
    repo.push(".git");
    assert!(repo.exists());

    // remote.origin.url is a read-only git:// URL
    {
        let command = ["git", "config", "remote.origin.url"];
        let exec = cmd::exec_in_dir(&command, "tests/init/example/tree/repo");
        if let Ok(x) = cmd::capture_stdout(exec) {
            let output = cmd::trim_stdout(&x);
            assert_eq!(output, "repos/example.git");
        } else {
            assert!(false, "unable to run 'git config remote.origin.url'");
        }
    }

    // remote.publish.url is a ssh push URL
    {
        let command = ["git", "config", "remote.publish.url"];
        let exec = cmd::exec_in_dir(&command, "tests/init/example/tree/repo");
        if let Ok(x) = cmd::capture_stdout(exec) {
            let output = cmd::trim_stdout(&x);
            assert_eq!(output, "git@github.com:user/example.git");
        } else {
            assert!(false, "unable to run 'git config remote.origin.url'");
        }
    }
}

}  // integration
