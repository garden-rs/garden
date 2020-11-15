use garden::cmd;


// Slow or filesystem/IO-heavy integration tests go in the slow namespace
// and are enabled by using "cargo test --features integration"
#[cfg(feature = "integration")]
mod slow {
    use std::path::Path;

    use anyhow::Result;

    use garden::cmd;

    /// Cleanup and create a bare repository for cloning
    fn setup(name: &str, path: &str) {
        let cmd = ["../integration/setup.sh", name];
        assert_eq!(0, cmd::status(cmd::exec_in_dir(&cmd, path).join()));
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
            "--chdir",
            "tests/tmp/clone",
            "--config",
            "tests/data/garden.yaml",
            "grow",
            "example/tree",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

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
            "--chdir",
            "tests/tmp/remotes",
            "--config",
            "tests/data/garden.yaml",
            "grow",
            "example/tree",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // remote.origin.url is a read-only git:// URL
        {
            let command = ["git", "config", "remote.origin.url"];
            let exec = cmd::exec_in_dir(
                &command, "tests/tmp/remotes/example/tree/repo");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());
            let output = cmd::trim_stdout(&capture.unwrap());
            assert!(
                output.ends_with("/tests/tmp/remotes/repos/example.git"),
                format!(
                    "{} does not end with {}",
                    output,
                    "/tests/tmp/clone/repos/example.git"
                )
            );
        }

        // remote.publish.url is a ssh push URL
        {
            let command = ["git", "config", "remote.publish.url"];
            let exec = cmd::exec_in_dir(
                &command, "tests/tmp/remotes/example/tree/repo");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());
            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!("git@github.com:user/example.git", output);
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
                "--chdir",
                "tests/tmp/symlinks",
                "--config",
                "tests/data/garden.yaml",
                "grow",
                "example/tree",
                "link",
                "example/link",
            ];
            assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        }

        // tests/tmp/symlinks/trees/example/repo exists
        {
            let repo = std::path::PathBuf::from("tests/tmp/symlinks/example/tree/repo/.git");
            assert!(repo.exists());
        }

        // tests/tmp/symlinks/link is a symlink pointing to example/tree/repo
        {
            let link = std::path::PathBuf::from("tests/tmp/symlinks/link");
            assert!(link.exists(), "tests/tmp/symlinks/link does not exist");
            assert!(link.read_link().is_ok());

            let target = link.read_link().unwrap();
            assert_eq!("example/tree/repo", target.to_string_lossy());
        }

        // tests/tmp/symlinks/example/link is a symlink pointing to tree/repo
        {
            let link = std::path::PathBuf::from("tests/tmp/symlinks/example/link");
            assert!(
                link.exists(),
                "tests/tmp/symlinks/example/link does not exist"
            );
            assert!(link.read_link().is_ok());

            let target = link.read_link().unwrap();
            assert_eq!("tree/repo", target.to_string_lossy());
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
                "--chdir",
                "tests/tmp/gitconfig",
                "--config",
                "tests/data/garden.yaml",
                "grow",
                "example/tree",
            ];
            assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        }

        // remote.origin.annex-ignore is true
        {
            let command = ["git", "config", "remote.origin.annex-ignore"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/gitconfig/example/tree/repo");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());
            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!("true", output);
        }

        // user.name is "A U Thor"
        {
            let command = ["git", "config", "user.name"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/gitconfig/example/tree/repo");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());
            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!("A U Thor", output);
        }

        // user.email is "author@example.com"
        {
            let command = ["git", "config", "user.email"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/gitconfig/example/tree/repo");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());
            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!("author@example.com", output);
        }

        teardown("tests/tmp/gitconfig");
    }


    /// `garden add` adds an empty repository
    #[test]
    fn add_empty_repo() -> Result<()> {
        setup("add-empty-repo", "tests/tmp");

        // garden init in test/tmp/add-empty-repo
        let cmd = [
            "./target/debug/garden", "--chdir", "tests/tmp/add-empty-repo", "init",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        // Empty garden.yaml should be created
        assert!(Path::new("tests/tmp/add-empty-repo/garden.yaml").exists());

        // Create tests/tmp/add-empty-repo/repo{1,2}
        let cmd = ["git", "-C", "tests/tmp/add-empty-repo", "init", "repo1"];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        let cmd = ["git", "-C", "tests/tmp/add-empty-repo", "init", "repo2"];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // repo1 has two remotes: "origin" and "remote-1".
        // git remote add origin repo-1-url
        let cmd = [
            "git", "-C", "tests/tmp/add-empty-repo/repo1",
            "remote", "add", "origin", "repo-1-url"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        // git remote add remote-1 remote-1-url
        let cmd = [
            "git", "-C", "tests/tmp/add-empty-repo/repo1",
            "remote", "add", "remote-1", "remote-1-url"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // garden add repo1
        let cmd = [
            "./target/debug/garden", "--chdir", "tests/tmp/add-empty-repo",
            "add", "repo1"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        let path = Some(
            std::path::PathBuf::from("tests/tmp/add-empty-repo/garden.yaml")
        );

        // Load the configuration and assert that the remotes are configured.
        let cfg = garden::config::new(&path, "", false)?;
        assert_eq!(1, cfg.trees.len());
        assert_eq!("repo1", cfg.trees[0].get_name());
        assert_eq!(2, cfg.trees[0].remotes.len());
        assert_eq!("origin", cfg.trees[0].remotes[0].get_name());
        assert_eq!("repo-1-url", cfg.trees[0].remotes[0].get_expr());
        assert_eq!("remote-1", cfg.trees[0].remotes[1].get_name());
        assert_eq!("remote-1-url", cfg.trees[0].remotes[1].get_expr());

        // repo2 has two remotes: "remote-1" and "remote-2".
        // git remote add remote-1 remote-1-url
        let cmd = [
            "git", "-C", "tests/tmp/add-empty-repo/repo2",
            "remote", "add", "remote-1", "remote-1-url"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        // git remote add remote-2 remote-2-url
        let cmd = [
            "git", "-C", "tests/tmp/add-empty-repo/repo2",
            "remote", "add", "remote-2", "remote-2-url"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // garden add repo2
        let cmd = [
            "./target/debug/garden", "--chdir", "tests/tmp/add-empty-repo",
            "add", "repo2"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Load the configuration and assert that the remotes are configured.
        let cfg = garden::config::new(&path, "", false)?;
        assert_eq!(2, cfg.trees.len());  // Now we have two trees.
        assert_eq!("repo2", cfg.trees[1].get_name());
        assert_eq!(2, cfg.trees[1].remotes.len());
        assert_eq!("remote-1", cfg.trees[1].remotes[0].get_name());
        assert_eq!("remote-1-url", cfg.trees[1].remotes[0].get_expr());
        assert_eq!("remote-2", cfg.trees[1].remotes[1].get_name());
        assert_eq!("remote-2-url", cfg.trees[1].remotes[1].get_expr());

        // Verify that "garden add" will refresh the remote URLs
        // for existing entries.

        // Update repo1's origin url to repo-1-new-url.
        // git config remote.origin.url repo-1-new-url
        let cmd = [
            "git", "-C", "tests/tmp/add-empty-repo/repo1",
            "config", "remote.origin.url", "repo-1-new-url"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Update repo2's remote-2 url to remote-2-new-url.
        // git config remote.remote-2.url remote-2-new-url
        let cmd = [
            "git", "-C", "tests/tmp/add-empty-repo/repo2",
            "config", "remote.remote-2.url", "remote-2-new-url"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // garden add repo1 repo2
        let cmd = [
            "./target/debug/garden", "--chdir", "tests/tmp/add-empty-repo",
            "add", "repo1", "repo2"
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Load the configuration and assert that the remotes are configured.
        let cfg = garden::config::new(&path, "", false)?;
        assert_eq!(2, cfg.trees.len());
        assert_eq!("repo1", cfg.trees[0].get_name());
        assert_eq!(2, cfg.trees[0].remotes.len());
        assert_eq!("origin", cfg.trees[0].remotes[0].get_name());
        assert_eq!("repo-1-new-url", cfg.trees[0].remotes[0].get_expr());  // New value.
        assert_eq!("remote-1", cfg.trees[0].remotes[1].get_name());
        assert_eq!("remote-1-url", cfg.trees[0].remotes[1].get_expr());

        assert_eq!("repo2", cfg.trees[1].get_name());
        assert_eq!(2, cfg.trees[1].remotes.len());
        assert_eq!("remote-1", cfg.trees[1].remotes[0].get_name());
        assert_eq!("remote-1-url", cfg.trees[1].remotes[0].get_expr());
        assert_eq!("remote-2", cfg.trees[1].remotes[1].get_name());
        // New value.
        assert_eq!("remote-2-new-url", cfg.trees[1].remotes[1].get_expr());

        teardown("tests/tmp/add-empty-repo");

        Ok(())
    }


    /// `garden eval` evaluates ${GARDEN_CONFIG_DIR}
    #[test]
    fn eval_garden_config_dir() {
        setup("configdir", "tests/tmp");

        // garden eval ${GARDEN_CONFIG_DIR}
        {
            let cmd = [
                "./target/debug/garden",
                "--chdir",
                "tests/tmp/configdir",
                "--config",
                "tests/data/garden.yaml",
                "eval",
                "${GARDEN_CONFIG_DIR}",
            ];
            let exec = cmd::exec_cmd(&cmd);
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());
            let output = cmd::trim_stdout(&capture.unwrap());
            assert!(
                output.ends_with("/tests/data"),
                format!("{} does not end with /tests/data", output)
            );
        }

        teardown("tests/tmp/configdir");
    }

} // slow


/// Test eval behavior around the "--root" option
#[test]
fn eval_root_with_root() {
    let cmd = [
        "./target/debug/garden",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_ROOT}",
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
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_CONFIG_DIR}",
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
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "$ pwd",
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
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_ROOT}",
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
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        ".",
        "echo-dir",
        "echo-args",
        "echo-dir",
        "echo-args",
        "--",
        "d",
        "e",
        "f",
        "--",
        "g",
        "h",
        "i",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    // Repeated command names were used to operate on the tree twice.
    let msg = format!(
        "data\ngarden\n{}",
        "arguments -- a b c -- d e f -- g h i -- x y z"
    );
    assert_eq!(output, format!("{}\n{}", msg, msg));
}


/// Test dash-dash arguments in custom commands via "garden <custom> ..."
#[test]
fn cmd_dash_dash_arguments_custom() {
    let cmd = [
        "./target/debug/garden",
        "--chdir",
        "tests/data",
        "--quiet",
        "echo-args",
        ".",
        ".",
        "--",
        "d",
        "e",
        "f",
        "--",
        "g",
        "h",
        "i",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    // `. .` was used to operate on the tree twice.
    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(format!("{}\n{}", msg, msg), output);
}


/// Test "." default for custom "garden <command>" with no arguments
#[test]
fn cmd_dot_default_no_args() {
    let cmd = [
        "./target/debug/garden",
        "--quiet",
        "--chdir",
        "tests/data",
        "echo-dir",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());
    assert_eq!("data", output);
}


/// Test "." default for "garden <command>" with no arguments and echo
#[test]
fn cmd_dot_default_no_args_echo() {
    let cmd = [
        "./target/debug/garden",
        "--quiet",
        "--chdir",
        "tests/data",
        "echo-args",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(msg, output);
}


/// Test "." default for "garden <command>" with double-dash
#[test]
fn cmd_dot_default_double_dash() {
    let cmd = [
        "./target/debug/garden",
        "--quiet",
        "--chdir",
        "tests/data",
        "echo-args",
        "--",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(msg, output);

}


/// Test "." default for "garden <command>" with extra arguments
#[test]
fn cmd_dot_default_double_dash_args() {
    let cmd = [
        "./target/debug/garden",
        "--quiet",
        "--chdir",
        "tests/data",
        "echo-args",
        "--",
        "d",
        "e",
        "f",
        "--",
        "g",
        "h",
        "i",
    ];
    let exec = cmd::exec_cmd(&cmd);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());
    let output = cmd::trim_stdout(&capture.unwrap());

    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(msg, output);

}
