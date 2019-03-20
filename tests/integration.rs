extern crate subprocess;
extern crate garden;

#[cfg(test)]
mod integration {

use super::garden::cmd;

/// Cleanup and create a bare repository for cloning
fn setup(name: &str, path: &str) {
    let cmd = ["../integration/setup.sh", name];
    let exec = garden::cmd::exec_in_dir(&cmd, path);
    let exit_status = garden::cmd::status(exec.join());
    assert_eq!(exit_status, 0);
}

fn teardown(path: &str) {
    if let Err(err) = std::fs::remove_dir_all(path) {
        assert!(false, format!("unable to remove '{}': {}", path, err));
    }
}

/// `gdn init` clones repositories
#[test]
fn integration_gdn_init_clone() {
    setup("clone", "tests/tmp");

    // gdn init examples/tree
    let cmd = [
        "./target/debug/gdn",
        "--chdir", "./tests/tmp/clone",
        "--config", "../../integration/garden.yaml",
        "init", "example/tree",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let exit_status = garden::cmd::status(exec.join());
    assert_eq!(exit_status, 0);

    // Ensure the repository was created
    let mut repo = std::path::PathBuf::from("tests");
    assert!(repo.exists());
    // tests/tmp
    repo.push("tmp");
    assert!(repo.exists());
    // tests/tmp/clone/example
    repo.push("clone");
    assert!(repo.exists());
    // tests/tmp/clone/example
    repo.push("example");
    assert!(repo.exists());
    // tests/tmp/clone/example/tree
    repo.push("tree");
    assert!(repo.exists());
    // tests/tmp/clone/example/tree/repo
    repo.push("repo");
    assert!(repo.exists());
    // tests/tmp/clone/example/tree/repo/.git
    repo.push(".git");
    assert!(repo.exists());

    teardown("tests/tmp/clone");
}


/// `gdn init` sets up remotes
#[test]
fn gdn_init_remotes() {
    setup("remotes", "tests/tmp");

    // gdn init examples/tree
    let cmd = [
        "./target/debug/gdn",
        "--chdir", "./tests/tmp/remotes",
        "--config", "../../integration/garden.yaml",
        "init", "example/tree",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let exit_status = garden::cmd::status(exec.join());
    assert_eq!(exit_status, 0);

    // remote.origin.url is a read-only git:// URL
    {
        let command = ["git", "config", "remote.origin.url"];
        let exec = cmd::exec_in_dir(
            &command, "tests/tmp/remotes/example/tree/repo");
        let capture = cmd::capture_stdout(exec);
        assert!(capture.is_ok());
        let output = cmd::trim_stdout(&capture.unwrap());
        assert!(output.ends_with("/tests/tmp/remotes/repos/example.git"),
                format!("{} does not end with {}",
                        output, "/tests/tmp/clone/repos/example.git"));
    }

    // remote.publish.url is a ssh push URL
    {
        let command = ["git", "config", "remote.publish.url"];
        let exec = cmd::exec_in_dir(&command, "tests/tmp/remotes/example/tree/repo");
        let capture = cmd::capture_stdout(exec);
        assert!(capture.is_ok());
        let output = cmd::trim_stdout(&capture.unwrap());
        assert_eq!(output, "git@github.com:user/example.git");
    }

    teardown("tests/tmp/remotes");
}

/// `gdn init` creates symlinks
#[test]
fn integration_gdn_init_symlinks() {
    setup("symlinks", "tests/tmp");

    // gdn init examples/tree examples/symlink
    {
        let cmd = [
            "./target/debug/gdn",
            "--chdir", "./tests/tmp/symlinks",
            "--config", "../../integration/garden.yaml",
            "init", "example/tree", "link", "example/link",
        ];
        let exec = garden::cmd::exec_cmd(&cmd);
        let exit_status = garden::cmd::status(exec.join());
        assert_eq!(exit_status, 0);
    }

    // tests/tmp/symlinks/trees/example/repo exists
    {
        let repo = std::path::PathBuf::from(
            "tests/tmp/symlinks/example/tree/repo/.git");
        assert!(repo.exists());
    }

    // tests/tmp/symlinks/link is a symlink pointing to example/tree/repo
    {
        let link = std::path::PathBuf::from("tests/tmp/symlinks/link");
        assert!(link.exists(), "tests/tmp/symlinks/link does not exist");
        assert!(link.read_link().is_ok());

        let target = link.read_link().unwrap();
        assert_eq!(target.to_string_lossy(), "example/tree/repo");
    }

    // tests/tmp/symlinks/example/link is a symlink pointing to tree/repo
    {
        let link = std::path::PathBuf::from("tests/tmp/symlinks/example/link");
        assert!(link.exists(), "tests/tmp/symlinks/example/link does not exist");
        assert!(link.read_link().is_ok());

        let target = link.read_link().unwrap();
        assert_eq!(target.to_string_lossy(), "tree/repo");
    }

    teardown("tests/tmp/symlinks");
}

/// `gdn init` sets up git config settings
#[test]
fn integration_gdn_init_gitconfig() {
    setup("gitconfig", "tests/tmp");

    // gdn init examples/tree
    {
        let cmd = [
            "./target/debug/gdn",
            "--chdir", "./tests/tmp/gitconfig",
            "--config", "../../integration/garden.yaml",
            "init", "example/tree",
        ];
        let exec = garden::cmd::exec_cmd(&cmd);
        let exit_status = garden::cmd::status(exec.join());
        assert_eq!(exit_status, 0);
    }

    // remote.origin.annex-ignore is true
    {
        let command = ["git", "config", "remote.origin.annex-ignore"];
        let exec = cmd::exec_in_dir(
            &command, "tests/tmp/gitconfig/example/tree/repo");
        let capture = cmd::capture_stdout(exec);
        assert!(capture.is_ok());
        let output = cmd::trim_stdout(&capture.unwrap());
        assert_eq!(output, "true");
    }

    // user.name is "A U Thor"
    {
        let command = ["git", "config", "user.name"];
        let exec = cmd::exec_in_dir(
            &command, "tests/tmp/gitconfig/example/tree/repo");
        let capture = cmd::capture_stdout(exec);
        assert!(capture.is_ok());
        let output = cmd::trim_stdout(&capture.unwrap());
        assert_eq!(output, "A U Thor");
    }

    // user.email is "author@example.com"
    {
        let command = ["git", "config", "user.email"];
        let exec = cmd::exec_in_dir(
            &command, "tests/tmp/gitconfig/example/tree/repo");
        let capture = cmd::capture_stdout(exec);
        assert!(capture.is_ok());
        let output = cmd::trim_stdout(&capture.unwrap());
        assert_eq!(output, "author@example.com");
    }

    teardown("tests/tmp/gitconfig");
}

/// `gdn eval` evaluates ${GARDEN_CONFIG_DIR}
#[test]
fn integration_gdn_eval_garden_config_dir() {
    setup("configdir", "tests/tmp");

    // gdn eval ${GARDEN_CONFIG_DIR}
    {
        let cmd = [
            "./target/debug/gdn",
            "--chdir", "./tests/tmp/configdir",
            "--config", "../../integration/garden.yaml",
            "eval", "${GARDEN_CONFIG_DIR}",
        ];
        let exec = garden::cmd::exec_cmd(&cmd);
        let capture = cmd::capture_stdout(exec);
        assert!(capture.is_ok());
        let output = cmd::trim_stdout(&capture.unwrap());
        assert!(output.ends_with("/tests/integration"),
                format!("{} does not end with /tests/integration", output));
    }

    teardown("tests/tmp/configdir");
}

}  // integration
