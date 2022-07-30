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
            assert!(false, "unable to remove '{}': {}", path, err);
        }
    }

    /// `garden grow` clones repositories
    #[test]
    fn grow_clone() -> Result<()> {
        setup("clone", "tests/tmp");

        // garden grow examples/tree
        let cmd = [
            "./target/debug/garden",
            "--verbose",
            "--verbose",
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

        // The repository must have all branches by default.
        {
            let command = ["git", "rev-parse", "origin/dev", "origin/default"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/clone/example/tree/repo");
            assert_eq!(0, cmd::status(exec.join()));
        }

        teardown("tests/tmp/clone");

        Ok(())
    }

    /// `garden grow` can create shallow clones with depth: 1.
    #[test]
    fn grow_clone_shallow() -> Result<()> {
        setup("shallow", "tests/tmp");

        // garden grow examples/shallow
        let cmd = [
            "./target/debug/garden",
            "--verbose",
            "--verbose",
            "--chdir",
            "tests/tmp/shallow",
            "--config",
            "tests/data/garden.yaml",
            "grow",
            "example/shallow",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Ensure the repository was created
        let mut repo = std::path::PathBuf::from("tests");
        assert!(repo.exists());
        // tests/tmp
        repo.push("tmp");
        repo.push("shallow");
        repo.push("example");
        repo.push("tree");
        repo.push("shallow");
        // tests/tmp/shallow/example/tree/repo/.git
        repo.push(".git");
        assert!(repo.exists());

        // The repository must have the default branches.
        {
            let command = ["git", "rev-parse", "origin/default"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/shallow/example/tree/shallow");
            assert_eq!(0, cmd::status(exec.join()));
        }
        // The dev branch must exist because we cloned with --no-single-branch.
        {
            let command = ["git", "rev-parse", "origin/dev"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/shallow/example/tree/shallow");
            assert!(0 == cmd::status(exec.join()));
        }
        // Only one commit must be cloned because of "depth: 1".
        {
            let command = ["git", "rev-list", "HEAD"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/shallow/example/tree/shallow");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());

            let output = cmd::trim_stdout(&capture.unwrap());
            let lines = output.split("\n").collect::<Vec<&str>>();
            assert_eq!(lines.len(), 1); // One commit only!
        }

        teardown("tests/tmp/shallow");

        Ok(())
    }

    /// `garden grow` clones a single branch with "single-branch: true".
    #[test]
    fn grow_clone_single_branch() -> Result<()> {
        setup("single-branch", "tests/tmp");

        // garden grow examples/single-branch
        let cmd = [
            "./target/debug/garden",
            "--verbose",
            "--verbose",
            "--chdir",
            "tests/tmp/single-branch",
            "--config",
            "tests/data/garden.yaml",
            "grow",
            "example/single-branch",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Ensure the repository was created
        let mut repo = std::path::PathBuf::from("tests");
        assert!(repo.exists());
        // tests/tmp
        repo.push("tmp");
        repo.push("single-branch");
        repo.push("example");
        repo.push("tree");
        repo.push("single-branch");
        // tests/tmp/single-branch/example/tree/single-branch/.git
        repo.push(".git");
        assert!(repo.exists());

        // The repository must have the default branches.
        {
            let command = ["git", "rev-parse", "origin/default"];
            let exec = cmd::exec_in_dir(
                &command,
                "tests/tmp/single-branch/example/tree/single-branch",
            );
            assert_eq!(0, cmd::status(exec.join()));
        }
        // The dev branch must not exist because we cloned with --single-branch.
        {
            let command = ["git", "rev-parse", "origin/dev"];
            let exec = cmd::exec_in_dir(
                &command,
                "tests/tmp/single-branch/example/tree/single-branch",
            );
            assert!(0 != cmd::status(exec.join()));
        }
        // Only one commit must be cloned because of "depth: 1".
        {
            let command = ["git", "rev-list", "HEAD"];
            let exec = cmd::exec_in_dir(
                &command,
                "tests/tmp/single-branch/example/tree/single-branch",
            );
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());

            let output = cmd::trim_stdout(&capture.unwrap());
            let lines = output.split("\n").collect::<Vec<&str>>();
            assert_eq!(lines.len(), 1); // One commit only!
        }

        teardown("tests/tmp/single-branch");

        Ok(())
    }

    #[test]
    fn grow_branch_default() -> Result<()> {
        setup("branches", "tests/tmp");

        // garden grow default dev
        let cmd = [
            "./target/debug/garden",
            "--verbose",
            "--verbose",
            "--chdir",
            "tests/tmp/branches",
            "--config",
            "tests/data/branches.yaml",
            "grow",
            "default",
            "dev",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Ensure the repository was created
        let mut repo = std::path::PathBuf::from("tests");
        assert!(repo.exists());
        // tests/tmp
        repo.push("tmp");
        repo.push("branches");
        repo.push("default");
        // tests/tmp/branches/default/.git
        repo.push(".git");
        assert!(repo.exists());

        // The "default" repository must have a branch called "default" checked-out.
        {
            let command = ["git", "symbolic-ref", "--short", "HEAD"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/branches/default");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());

            let output = cmd::trim_stdout(&capture.unwrap());
            let lines = output.split("\n").collect::<Vec<&str>>();
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0], "default");
        }

        // The "dev" repository must have a branch called "dev" checked-out.
        {
            let command = ["git", "symbolic-ref", "--short", "HEAD"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/branches/dev");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());

            let output = cmd::trim_stdout(&capture.unwrap());
            let lines = output.split("\n").collect::<Vec<&str>>();
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0], "dev");
        }
        // The origin/dev and origin/default branches must exist because we cloned with
        // --no-single-branch.
        {
            let command = ["git", "rev-parse", "origin/default"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/branches/default");
            assert!(0 == cmd::status(exec.join()));

            let exec = cmd::exec_in_dir(&command, "tests/tmp/branches/dev");
            assert!(0 == cmd::status(exec.join()));
        }
        {
            let command = ["git", "rev-parse", "origin/dev"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/branches/default");
            assert!(0 == cmd::status(exec.join()));

            let exec = cmd::exec_in_dir(&command, "tests/tmp/branches/dev");
            assert!(0 == cmd::status(exec.join()));
        }

        teardown("tests/tmp/branches");

        Ok(())
    }

    /// This creates bare repositories based on the "bare.git" naming convention.
    /// The configuration does not specify "bare: true".
    #[test]
    fn grow_bare_repo() -> Result<()> {
        setup("grow-bare-repo", "tests/tmp");

        // garden grow bare.git
        let cmd = [
            "./target/debug/garden",
            "--verbose",
            "--verbose",
            "--chdir",
            "tests/tmp/grow-bare-repo",
            "--config",
            "tests/data/bare.yaml",
            "grow",
            "bare.git",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Ensure the repository was created
        // tests/tmp/grow-bare-repo/bare.git
        let mut repo = std::path::PathBuf::from("tests");
        repo.push("tmp");
        repo.push("grow-bare-repo");
        repo.push("bare.git");
        assert!(repo.exists());

        {
            let command = ["git", "rev-parse", "default"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/grow-bare-repo/bare.git");
            assert_eq!(0, cmd::status(exec.join()));
        }
        // The dev branch must exist because we cloned with --no-single-branch.
        {
            let command = ["git", "rev-parse", "dev"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/grow-bare-repo/bare.git");
            assert_eq!(0, cmd::status(exec.join()));
        }
        // The repository must be bare.
        {
            let command = ["git", "config", "core.bare"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/grow-bare-repo/bare.git");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());

            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!(output, String::from("true"));
        }

        teardown("tests/tmp/grow-bare-repo");

        Ok(())
    }

    /// This creates bare repositories using the "bare: true" configuration.
    #[test]
    fn grow_bare_repo_with_config() -> Result<()> {
        setup("grow-bare-repo-config", "tests/tmp");

        // garden grow bare.git
        let cmd = [
            "./target/debug/garden",
            "--verbose",
            "--verbose",
            "--chdir",
            "tests/tmp/grow-bare-repo-config",
            "--config",
            "tests/data/bare.yaml",
            "grow",
            "bare",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Ensure the repository was created
        // tests/tmp/grow-bare-repo-config/bare
        let mut repo = std::path::PathBuf::from("tests");
        repo.push("tmp");
        repo.push("grow-bare-repo-config");
        repo.push("bare");
        assert!(repo.exists());

        {
            let command = ["git", "rev-parse", "default"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/grow-bare-repo-config/bare");
            assert_eq!(0, cmd::status(exec.join()));
        }
        // The dev branch must exist because we cloned with --no-single-branch.
        {
            let command = ["git", "rev-parse", "dev"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/grow-bare-repo-config/bare");
            assert_eq!(0, cmd::status(exec.join()));
        }
        // The repository must be bare.
        {
            let command = ["git", "config", "core.bare"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/grow-bare-repo-config/bare");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());

            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!(output, String::from("true"));
        }

        teardown("tests/tmp/grow-bare-repo-config");

        Ok(())
    }

    /// `garden grow` sets up remotes
    #[test]
    fn grow_remotes() {
        setup("remotes", "tests/tmp");

        // garden grow examples/tree
        let cmd = [
            "./target/debug/garden",
            "--verbose",
            "--verbose",
            "--chdir",
            "tests/tmp/remotes",
            "--config",
            "tests/data/garden.yaml",
            "grow",
            "example/tree",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // remote.origin.url is a read-only https:// URL
        {
            let command = ["git", "config", "remote.origin.url"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/remotes/example/tree/repo");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());
            let output = cmd::trim_stdout(&capture.unwrap());
            assert!(
                output.ends_with("/tests/tmp/remotes/repos/example.git"),
                "{} does not end with {}",
                output,
                "/tests/tmp/clone/repos/example.git"
            );
        }

        // remote.publish.url is a ssh push URL
        {
            let command = ["git", "config", "remote.publish.url"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/remotes/example/tree/repo");
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());
            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!("git@github.com:user/example.git", output);
        }

        teardown("tests/tmp/remotes");
    }

    /// `garden grow` creates symlinks
    #[test]
    fn grow_symlinks() {
        setup("symlinks", "tests/tmp");

        // garden grow examples/tree examples/link
        {
            let cmd = [
                "./target/debug/garden",
                "--verbose",
                "--verbose",
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

    /// `garden grow` sets up git config settings
    #[test]
    fn grow_gitconfig() {
        setup("gitconfig", "tests/tmp");

        // garden grow examples/tree
        {
            let cmd = [
                "./target/debug/garden",
                "--verbose",
                "--verbose",
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

    /// This creates a worktree
    #[test]
    fn grow_worktree_and_parent() -> Result<()> {
        setup("grow-worktree-and-parent", "tests/tmp");

        // garden grow dev
        let cmd = [
            "./target/debug/garden",
            "--verbose",
            "--verbose",
            "--chdir",
            "tests/tmp/grow-worktree-and-parent",
            "--config",
            "tests/data/worktree.yaml",
            "grow",
            "dev",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Ensure the repository was created
        let mut repo = std::path::PathBuf::from("tests");
        // tests/tmp/grow-bare-repo-config/default
        repo.push("tmp");
        repo.push("grow-worktree-and-parent");
        repo.push("default");
        repo.push(".git");
        assert!(repo.exists());

        // tests/tmp/grow-bare-repo-config/dev
        repo.pop();
        repo.pop();
        repo.push("dev");
        repo.push(".git");
        assert!(repo.exists());

        {
            let command = ["git", "rev-parse", "default"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/grow-worktree-and-parent/default");
            assert_eq!(0, cmd::status(exec.join()));
        }
        // The dev branch must exist because we cloned with --no-single-branch.
        {
            let command = ["git", "rev-parse", "dev"];
            let exec = cmd::exec_in_dir(&command, "tests/tmp/grow-worktree-and-parent/dev");
            assert_eq!(0, cmd::status(exec.join()));
        }

        // Ensure that the "echo" command is available from the child worktree.
        {
            let command = [
                "./target/debug/garden",
                "--chdir",
                "tests/tmp/grow-worktree-and-parent",
                "--config",
                "tests/data/worktree.yaml",
                "echo",
                "dev",
                "--",
                "hello",
            ];
            let exec = cmd::exec_cmd(&command);
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());

            // The "echo" command is: echo ${TREE_NAME} "$@"
            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!("dev hello", output);
        }

        // Ensure that the "echo" command is available from the parent worktree.
        {
            let command = [
                "./target/debug/garden",
                "--chdir",
                "tests/tmp/grow-worktree-and-parent",
                "--config",
                "tests/data/worktree.yaml",
                "echo",
                "default",
                "--",
                "hello",
            ];
            let exec = cmd::exec_cmd(&command);
            let capture = cmd::capture_stdout(exec);
            assert!(capture.is_ok());

            // The "echo" command is: echo ${TREE_NAME} "$@"
            let output = cmd::trim_stdout(&capture.unwrap());
            assert_eq!("default hello", output);
        }

        teardown("tests/tmp/grow-worktree-and-parent");

        Ok(())
    }

    /// `garden plant` adds an empty repository
    #[test]
    fn plant_empty_repo() -> Result<()> {
        setup("plant-empty-repo", "tests/tmp");

        // garden plant in test/tmp/plant-empty-repo
        let cmd = [
            "./target/debug/garden",
            "--chdir",
            "tests/tmp/plant-empty-repo",
            "init",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        // Empty garden.yaml should be created
        assert!(Path::new("tests/tmp/plant-empty-repo/garden.yaml").exists());

        // Create tests/tmp/plant-empty-repo/repo{1,2}
        let cmd = ["git", "-C", "tests/tmp/plant-empty-repo", "init", "repo1"];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        let cmd = ["git", "-C", "tests/tmp/plant-empty-repo", "init", "repo2"];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // repo1 has two remotes: "origin" and "remote-1".
        // git remote add origin repo-1-url
        let cmd = [
            "git",
            "-C",
            "tests/tmp/plant-empty-repo/repo1",
            "remote",
            "add",
            "origin",
            "repo-1-url",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        // git remote add remote-1 remote-1-url
        let cmd = [
            "git",
            "-C",
            "tests/tmp/plant-empty-repo/repo1",
            "remote",
            "add",
            "remote-1",
            "remote-1-url",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // garden plant repo1
        let cmd = [
            "./target/debug/garden",
            "--chdir",
            "tests/tmp/plant-empty-repo",
            "plant",
            "repo1",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        let path = Some(std::path::PathBuf::from(
            "tests/tmp/plant-empty-repo/garden.yaml",
        ));

        // Load the configuration and assert that the remotes are configured.
        let cfg = garden::config::new(&path, "", 0, None)?;
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
            "git",
            "-C",
            "tests/tmp/plant-empty-repo/repo2",
            "remote",
            "add",
            "remote-1",
            "remote-1-url",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        // git remote add remote-2 remote-2-url
        let cmd = [
            "git",
            "-C",
            "tests/tmp/plant-empty-repo/repo2",
            "remote",
            "add",
            "remote-2",
            "remote-2-url",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // garden add repo2
        let cmd = [
            "./target/debug/garden",
            "--chdir",
            "tests/tmp/plant-empty-repo",
            "plant",
            "repo2",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Load the configuration and assert that the remotes are configured.
        let cfg = garden::config::new(&path, "", 0, None)?;
        assert_eq!(2, cfg.trees.len()); // Now we have two trees.
        assert_eq!("repo2", cfg.trees[1].get_name());
        assert_eq!(2, cfg.trees[1].remotes.len());
        assert_eq!("remote-1", cfg.trees[1].remotes[0].get_name());
        assert_eq!("remote-1-url", cfg.trees[1].remotes[0].get_expr());
        assert_eq!("remote-2", cfg.trees[1].remotes[1].get_name());
        assert_eq!("remote-2-url", cfg.trees[1].remotes[1].get_expr());

        // Verify that "garden plant" will refresh the remote URLs
        // for existing entries.

        // Update repo1's origin url to repo-1-new-url.
        // git config remote.origin.url repo-1-new-url
        let cmd = [
            "git",
            "-C",
            "tests/tmp/plant-empty-repo/repo1",
            "config",
            "remote.origin.url",
            "repo-1-new-url",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Update repo2's remote-2 url to remote-2-new-url.
        // git config remote.remote-2.url remote-2-new-url
        let cmd = [
            "git",
            "-C",
            "tests/tmp/plant-empty-repo/repo2",
            "config",
            "remote.remote-2.url",
            "remote-2-new-url",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // garden plant repo1 repo2
        let cmd = [
            "./target/debug/garden",
            "--chdir",
            "tests/tmp/plant-empty-repo",
            "plant",
            "repo1",
            "repo2",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // Load the configuration and assert that the remotes are configured.
        let cfg = garden::config::new(&path, "", 0, None)?;
        assert_eq!(2, cfg.trees.len());
        assert_eq!("repo1", cfg.trees[0].get_name());
        assert_eq!(2, cfg.trees[0].remotes.len());
        assert_eq!("origin", cfg.trees[0].remotes[0].get_name());
        assert_eq!("repo-1-new-url", cfg.trees[0].remotes[0].get_expr()); // New value.
        assert_eq!("remote-1", cfg.trees[0].remotes[1].get_name());
        assert_eq!("remote-1-url", cfg.trees[0].remotes[1].get_expr());

        assert_eq!("repo2", cfg.trees[1].get_name());
        assert_eq!(2, cfg.trees[1].remotes.len());
        assert_eq!("remote-1", cfg.trees[1].remotes[0].get_name());
        assert_eq!("remote-1-url", cfg.trees[1].remotes[0].get_expr());
        assert_eq!("remote-2", cfg.trees[1].remotes[1].get_name());
        // New value.
        assert_eq!("remote-2-new-url", cfg.trees[1].remotes[1].get_expr());

        teardown("tests/tmp/plant-empty-repo");

        Ok(())
    }

    /// `garden plant` detects bare repositories.
    #[test]
    fn plant_bare_repo() -> Result<()> {
        setup("plant-bare-repo", "tests/tmp");

        // garden plant in test/tmp/plant-bare-repo
        let cmd = [
            "./target/debug/garden",
            "--chdir",
            "tests/tmp/plant-bare-repo",
            "init",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));
        // Empty garden.yaml should be created
        assert!(Path::new("tests/tmp/plant-bare-repo/garden.yaml").exists());

        // Create tests/tmp/plant-bare-repo/repo.git
        let cmd = [
            "git",
            "-C",
            "tests/tmp/plant-bare-repo",
            "init",
            "--bare",
            "repo.git",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        // garden plant repo.git
        let cmd = [
            "./target/debug/garden",
            "--chdir",
            "tests/tmp/plant-bare-repo",
            "plant",
            "repo.git",
        ];
        assert_eq!(0, cmd::status(cmd::exec_cmd(&cmd).join()));

        let path = Some(std::path::PathBuf::from(
            "tests/tmp/plant-bare-repo/garden.yaml",
        ));

        // Load the configuration and assert that the remotes are configured.
        let cfg = garden::config::new(&path, "", 0, None)?;
        assert_eq!(1, cfg.trees.len());
        assert_eq!("repo.git", cfg.trees[0].get_name());

        // The generated config must have "bare: true" configured.
        assert!(cfg.trees[0].is_bare_repository);

        teardown("tests/tmp/plant-bare-repo");
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
                "{} does not end with /tests/data",
                output
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
