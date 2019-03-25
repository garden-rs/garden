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

/// `garden init` clones repositories
#[test]
fn int_init_clone() {
    setup("clone", "tests/tmp");

    // garden init examples/tree
    let cmd = [
        "./target/debug/garden",
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


/// `garden init` sets up remotes
#[test]
fn int_init_remotes() {
    setup("remotes", "tests/tmp");

    // garden init examples/tree
    let cmd = [
        "./target/debug/garden",
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

/// `garden init` creates symlinks
#[test]
fn int_init_symlinks() {
    setup("symlinks", "tests/tmp");

    // garden init examples/tree examples/symlink
    {
        let cmd = [
            "./target/debug/garden",
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

/// `garden init` sets up git config settings
#[test]
fn int_init_gitconfig() {
    setup("gitconfig", "tests/tmp");

    // garden init examples/tree
    {
        let cmd = [
            "./target/debug/garden",
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

/// `garden eval` evaluates ${GARDEN_CONFIG_DIR}
#[test]
fn int_eval_garden_config_dir() {
    setup("configdir", "tests/tmp");

    // garden eval ${GARDEN_CONFIG_DIR}
    {
        let cmd = [
            "./target/debug/garden",
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

/// Test dash-dash arguments in custom commands via "garden cmd ..."
#[test]
fn int_cmd_dash_dash_arguments() {
    let cmd = [
        "./target/debug/garden",
        "--chdir", "./tests/integration",
        "--quiet",
        "cmd", ".",
        "echo-dir", "echo-args",
        "echo-dir", "echo-args",
        "--", "d", "e", "f",
        "--", "g", "h", "i",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    // Repeated command names were used to operate on the tree twice.
    let msg = format!("integration\ngarden\n{}",
                      "arguments -- a b c -- d e f -- g h i -- x y z");
    assert_eq!(output, format!("{}\n{}", msg, msg));
}

/// Test dash-dash arguments in custom commands via "garden <custom> ..."
#[test]
fn int_cmd_dash_dash_arguments_custom() {
    let cmd = [
        "./target/debug/garden",
        "--chdir", "./tests/integration",
        "--quiet",
        "echo-args", ".", ".",
        "--", "d", "e", "f",
        "--", "g", "h", "i",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    // `. .` was used to operate on the tree twice.
    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(output, format!("{}\n{}", msg, msg));
}

/// Test "." default for custom "garden <command>" with no arguments
#[test]
fn int_cmd_dot_default_no_args() {
    let cmd = [
        "./target/debug/garden", "--quiet", "--chdir", "./tests/integration",
        "echo-dir",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());
    assert_eq!(output, "integration");
}

/// Test "." default for "garden <command>" with no arguments and echo
#[test]
fn int_cmd_dot_default_no_args_echo() {
    let cmd = [
        "./target/debug/garden", "--quiet", "--chdir", "./tests/integration",
        "echo-args",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(output, msg);

}


/// Test "." default for "garden <command>" with double-dash
#[test]
fn int_cmd_dot_default_double_dash() {
    let cmd = [
        "./target/debug/garden", "--quiet", "--chdir", "./tests/integration",
        "echo-args", "--",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(output, msg);

}

/// Test "." default for "garden <command>" with extra arguments
#[test]
fn int_cmd_dot_default_double_dash_args() {
    let cmd = [
        "./target/debug/garden", "--quiet", "--chdir", "./tests/integration",
        "echo-args", "--", "d", "e", "f", "--", "g", "h", "i",
    ];
    let exec = garden::cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(output, msg);

}


}  // integration
