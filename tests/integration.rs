extern crate subprocess;
extern crate garden;

#[cfg(test)]
mod integration {

use super::garden::cmd;

#[test]
fn garden_init() {
    // First, cleanup existing repos from any previous runs
    if std::path::PathBuf::from("tests/init/example").exists() {
        if let Err(err) = std::fs::remove_dir_all("tests/init/example") {
            assert!(false, "unable to remove tests/init/example: {}", err);
        }
    }

    let cmd = [
        "./target/debug/garden",
        "--chdir", "./tests/init",
        "init", "example/vx",
    ];
    let exec = garden::cmd::exec_in_dir(&cmd, ".");
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
    // tests/init/example/vx
    repo.push("vx");
    assert!(repo.exists());
    // tests/init/example/vx/repo
    repo.push("repo");
    assert!(repo.exists());
    // tests/init/example/vx/repo/.git
    repo.push(".git");
    assert!(repo.exists());

    // remote.origin.url is a read-only git:// URL
    {
        let command = ["git", "config", "remote.origin.url"];
        let exec = cmd::exec_in_dir(&command, "tests/init/example/vx/repo");
        if let Ok(x) = cmd::capture_stdout(exec) {
            let output = cmd::trim_stdout(&x);
            assert_eq!(output, "git://github.com/davvid/vx.git");
        } else {
            assert!(false, "unable to run 'git config remote.origin.url'");
        }
    }

    // remote.publish.url is a ssh push URL
    {
        let command = ["git", "config", "remote.publish.url"];
        let exec = cmd::exec_in_dir(&command, "tests/init/example/vx/repo");
        if let Ok(x) = cmd::capture_stdout(exec) {
            let output = cmd::trim_stdout(&x);
            assert_eq!(output, "git@github.com:davvid/vx.git");
        } else {
            assert!(false, "unable to run 'git config remote.origin.url'");
        }
    }
}

}  // integration
