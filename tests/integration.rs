use garden::cmd;


// Slow or filesystem/IO-heavy integration tests go in the slow namespace
// and are enabled by using "cargo test --features integration"
#[cfg(feature = "integration")]
mod slow {

use garden::cmd;

/// Cleanup and create a bare repository for cloning
fn setup(name: &str, path: &str) {
    let cmd = ["../integration/setup.sh", name];
    let exec = cmd::exec_in_dir(&cmd, path);
    let exit_status = cmd::status(exec.join());
    assert_eq!(exit_status, 0);
}


fn teardown(path: &str) {
    if let Err(err) = std::fs::remove_dir_all(path) {
        assert!(false, format!("unable to remove '{}': {}", path, err));
    }
}


/// `garden init` clones repositories
#[test]
fn grow_clone() {
    setup("clone", "tests/tmp");

    // garden init examples/tree
    let cmd = [
        "./target/debug/garden",
        "--chdir", "tests/tmp/clone",
        "--config", "tests/data/garden.yaml",
        "grow", "example/tree",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let exit_status = cmd::status(exec.join());
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
fn grow_remotes() {
    setup("remotes", "tests/tmp");

    // garden init examples/tree
    let cmd = [
        "./target/debug/garden",
        "--chdir", "tests/tmp/remotes",
        "--config", "tests/data/garden.yaml",
        "grow", "example/tree",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let exit_status = cmd::status(exec.join());
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
fn grow_symlinks() {
    setup("symlinks", "tests/tmp");

    // garden init examples/tree examples/symlink
    {
        let cmd = [
            "./target/debug/garden",
            "--chdir", "tests/tmp/symlinks",
            "--config", "tests/data/garden.yaml",
            "grow", "example/tree", "link", "example/link",
        ];
        let exec = cmd::exec_cmd(&cmd);
        let exit_status = cmd::status(exec.join());
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
fn grow_gitconfig() {
    setup("gitconfig", "tests/tmp");

    // garden init examples/tree
    {
        let cmd = [
            "./target/debug/garden",
            "--chdir", "tests/tmp/gitconfig",
            "--config", "tests/data/garden.yaml",
            "grow", "example/tree",
        ];
        let exec = cmd::exec_cmd(&cmd);
        let exit_status = cmd::status(exec.join());
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
fn eval_garden_config_dir() {
    setup("configdir", "tests/tmp");

    // garden eval ${GARDEN_CONFIG_DIR}
    {
        let cmd = [
            "./target/debug/garden",
            "--chdir", "tests/tmp/configdir",
            "--config", "tests/data/garden.yaml",
            "eval", "${GARDEN_CONFIG_DIR}",
        ];
        let exec = cmd::exec_cmd(&cmd);
        let capture = cmd::capture_stdout(exec);
        assert!(capture.is_ok());
        let output = cmd::trim_stdout(&capture.unwrap());
        assert!(output.ends_with("/tests/data"),
                format!("{} does not end with /tests/data", output));
    }

    teardown("tests/tmp/configdir");
}

}  // slow


/// Test eval behavior around the "--root" option
#[test]
fn eval_root_with_root() {
    let cmd = [
        "./target/debug/garden",
        "--config", "tests/data/garden.yaml",
        "--root", "tests/tmp",
        "eval", "${GARDEN_ROOT}",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());

    let output = cmd::trim_stdout(&capture.unwrap());
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}


/// Test eval ${GARDEN_CONFIG_DIR} behavior with both "--root" and "--chdir"
#[test]
fn eval_config_dir_with_chdir_and_root() {
    let cmd = [
        "./target/debug/garden",
        "--chdir", "tests/tmp",
        "--config", "tests/data/garden.yaml",
        "--root", "tests/tmp",
        "eval", "${GARDEN_CONFIG_DIR}",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());

    let output = cmd::trim_stdout(&capture.unwrap());
    assert!(output.ends_with("/tests/data"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}


/// Test pwd with both "--root" and "--chdir"
#[test]
fn eval_exec_pwd_with_root_and_chdir() {
    let cmd = [
        "./target/debug/garden",
        "--chdir", "tests/tmp",
        "--config", "tests/data/garden.yaml",
        "--root", "tests/tmp",
        "eval", "$ pwd",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());

    let output = cmd::trim_stdout(&capture.unwrap());
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}


/// Test ${GARDEN_ROOT} with both "--root" and "--chdir"
#[test]
fn eval_root_with_root_and_chdir() {
    let cmd = [
        "./target/debug/garden",
        "--chdir", "tests/tmp",
        "--config", "tests/data/garden.yaml",
        "--root", "tests/tmp",
        "eval", "${GARDEN_ROOT}",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());

    let output = cmd::trim_stdout(&capture.unwrap());
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}


/// Test dash-dash arguments in custom commands via "garden cmd ..."
#[test]
fn cmd_dash_dash_arguments() {
    let cmd = [
        "./target/debug/garden",
        "--chdir", "tests/data",
        "--quiet",
        "cmd", ".",
        "echo-dir", "echo-args",
        "echo-dir", "echo-args",
        "--", "d", "e", "f",
        "--", "g", "h", "i",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    // Repeated command names were used to operate on the tree twice.
    let msg = format!("data\ngarden\n{}",
                      "arguments -- a b c -- d e f -- g h i -- x y z");
    assert_eq!(output, format!("{}\n{}", msg, msg));
}


/// Test dash-dash arguments in custom commands via "garden <custom> ..."
#[test]
fn cmd_dash_dash_arguments_custom() {
    let cmd = [
        "./target/debug/garden",
        "--chdir", "tests/data",
        "--quiet",
        "echo-args", ".", ".",
        "--", "d", "e", "f",
        "--", "g", "h", "i",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    // `. .` was used to operate on the tree twice.
    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(output, format!("{}\n{}", msg, msg));
}


/// Test "." default for custom "garden <command>" with no arguments
#[test]
fn cmd_dot_default_no_args() {
    let cmd = [
        "./target/debug/garden", "--quiet", "--chdir", "tests/data",
        "echo-dir",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());
    assert_eq!(output, "data");
}


/// Test "." default for "garden <command>" with no arguments and echo
#[test]
fn cmd_dot_default_no_args_echo() {
    let cmd = [
        "./target/debug/garden", "--quiet", "--chdir", "tests/data",
        "echo-args",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(output, msg);

}


/// Test "." default for "garden <command>" with double-dash
#[test]
fn cmd_dot_default_double_dash() {
    let cmd = [
        "./target/debug/garden", "--quiet", "--chdir", "tests/data",
        "echo-args", "--",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(output, msg);

}


/// Test "." default for "garden <command>" with extra arguments
#[test]
fn cmd_dot_default_double_dash_args() {
    let cmd = [
        "./target/debug/garden", "--quiet", "--chdir", "tests/data",
        "echo-args", "--", "d", "e", "f", "--", "g", "h", "i",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(output, msg);

}
