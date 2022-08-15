use garden::cmd;

use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;
use function_name::named;

use std::process::Command;

/// Execute the "garden" command with the specified arguments.
fn exec_garden(args: &[&str]) -> Result<()> {
    let mut exec = Command::cargo_bin("garden").expect("garden not found");
    exec.args(args);

    assert!(exec.status().expect("garden returned an error").success());
    Ok(())
}

/// Execute a command and ensure that exit status 0 is returned. Return the Exec object.
fn garden_capture(args: &[&str]) -> String {
    let mut exec = Command::cargo_bin("garden").expect("garden not found");
    exec.args(args);

    let capture = exec.output();
    assert!(capture.is_ok());

    let utf8_result = String::from_utf8(capture.unwrap().stdout);
    assert!(utf8_result.is_ok());

    utf8_result.unwrap().trim_end().into()
}

/// Execute a command and ensure that the exit status is returned.
fn assert_cmd_status(cmd: &[&str], directory: &str, status: i32) {
    let exec = cmd::exec_in_dir(&cmd, directory);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());

    match capture.unwrap().exit_status {
        subprocess::ExitStatus::Exited(val) => {
            assert_eq!(val as i32, status);
        },
        subprocess::ExitStatus::Signaled(val) => {
            assert_eq!(val as i32, status);
        },
        subprocess::ExitStatus::Other(val) => {
            assert_eq!(val, status);
        },
        subprocess::ExitStatus::Undetermined => {
            assert!(false, "undetermined exit status");
        },
    }
}

/// Execute a command and ensure that exit status 0 is returned.
fn assert_cmd(cmd: &[&str], directory: &str) {
    assert_cmd_status(cmd, directory, 0);
}

/// Execute a command and ensure that exit status 0 is returned. Return the Exec object.
fn assert_cmd_capture(cmd: &[&str], directory: &str) -> String {
    let exec = cmd::exec_in_dir(&cmd, directory);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());

    cmd::trim_stdout(&capture.unwrap())
}

/// Assert that the specified path exists.
fn assert_path(path: &str) {
    let pathbuf = std::path::PathBuf::from(path);
    assert!(pathbuf.exists());
}

/// Assert that the specified path is a Git worktree.
fn assert_git_worktree(path: &str) {
    assert_path(&format!("{}/.git", path));
}

/// Assert that the Git ref exists in the specified repository.
fn assert_ref(repository: &str, refname: &str) {
    let cmd = ["git", "rev-parse", "--quiet", "--verify", &refname];
    assert_cmd(&cmd, &repository);
}

/// Assert that the Git ref does not exist in the specified repository.
fn assert_ref_missing(repository: &str, refname: &str) {
    let cmd = ["git", "rev-parse", "--quiet", "--verify", &refname];
    assert_cmd_status(&cmd, &repository, 1);
}

/// Cleanup and create a bare repository for cloning
fn setup(name: &str, path: &str) {
    let cmd = ["../integration/setup.sh", name];
    assert_cmd(&cmd, path);
}

fn teardown(path: &str) {
    if let Err(err) = std::fs::remove_dir_all(path) {
        assert!(false, "unable to remove '{}': {}", path, err);
    }
}

/// Provide a bare repository fixture for the current test.
struct BareRepoFixture<'a> {
    name: &'a str,
}

impl<'a> BareRepoFixture<'a> {
    /// Create the test bare repository.
    fn new(name: &'a str) -> Self {
        setup(name, "tests/tmp");

        Self { name }
    }

    /// Return the temporary directory for the current test.
    fn root(&self) -> String {
        format!("tests/tmp/{}", self.name)
    }

    /// Return a path relative to the temporary directory for the current test.
    fn path(&self, path: &str) -> String {
        let fixture_path = format!("{}/{}", self.root(), path);
        assert_path(&fixture_path);

        fixture_path
    }

    /// Asserts that the path is a Git worktree.
    /// Returns the path to the specified worktree.
    fn worktree(&self, path: &str) -> String {
        let worktree = self.path(path);
        assert_git_worktree(&worktree);

        worktree
    }
}

impl Drop for BareRepoFixture<'_> {
    /// Teardown the test repository.
    fn drop(&mut self) {
        teardown(&self.root());
    }
}

/// `garden grow` clones repositories
#[test]
#[named]
fn grow_clone() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
    ])?;

    // A repository was created.
    let worktree = fixture.worktree("example/tree/repo");
    // The repository has all branches.
    assert_ref(&worktree, "origin/default");
    assert_ref(&worktree, "origin/dev");

    Ok(())
}

/// `garden grow` can create shallow clones with depth: 1.
#[test]
#[named]
fn grow_clone_shallow() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/shallow
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/shallow",
    ])?;

    // A repository was created.
    let worktree = fixture.worktree("example/tree/shallow");
    // The repository has all branches.
    assert_ref(&worktree, "origin/default");
    assert_ref(&worktree, "origin/dev");

    // Only one commit must be cloned because of "depth: 1".
    let cmd = ["git", "rev-list", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree);
    let lines = output.split("\n").collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1); // One commit only!

    Ok(())
}

/// `garden grow` clones a single branch with "single-branch: true".
#[test]
#[named]
fn grow_clone_single_branch() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/single-branch
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/single-branch",
    ])?;

    // A repository was created.
    let worktree = fixture.worktree("example/tree/single-branch");

    // The repository must have the default branch.
    assert_ref(&worktree, "origin/default");
    // The dev branch must *not* exist because we cloned with --single-branch.
    assert_ref_missing(&worktree, "origin/dev");

    // Only one commit must be cloned because of "depth: 1".
    let cmd = ["git", "rev-list", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree);
    let lines = output.split("\n").collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1); // One commit only!

    Ok(())
}

#[test]
#[named]
fn grow_branch_default() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow default dev
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/branches.yaml",
        "grow",
        "default",
        "dev",
    ])?;

    // Ensure the repositories were created.
    let worktree_default = fixture.worktree("default");
    let worktree_dev = fixture.worktree("dev");

    // The "default" repository must have a branch called "default" checked-out.
    let cmd = ["git", "symbolic-ref", "--short", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree_default);
    let lines = output.split("\n").collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "default");

    // The "dev" repository must have a branch called "dev" checked-out.
    let cmd = ["git", "symbolic-ref", "--short", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree_dev);
    let lines = output.split("\n").collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "dev");

    // The origin/dev and origin/default branches must exist because we cloned with
    // --no-single-branch.
    assert_ref(&worktree_default, "origin/default");
    assert_ref(&worktree_default, "origin/dev");

    assert_ref(&worktree_dev, "origin/default");
    assert_ref(&worktree_dev, "origin/dev");

    Ok(())
}

/// This creates bare repositories based on the "bare.git" naming convention.
/// The configuration does not specify "bare: true".
#[test]
#[named]
fn grow_bare_repo() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow bare.git
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/bare.yaml",
        "grow",
        "bare.git",
    ])?;

    // A repository was created.
    let bare_repo = fixture.path("bare.git");

    // The all branches must exist because we cloned with --no-single-branch.
    assert_ref(&bare_repo, "default");
    assert_ref(&bare_repo, "dev");

    // The repository must be bare.
    let cmd = ["git", "config", "--bool", "core.bare"];
    let output = assert_cmd_capture(&cmd, &bare_repo);
    assert_eq!(output, String::from("true"));

    Ok(())
}

/// This creates bare repositories using the "bare: true" configuration.
#[test]
#[named]
fn grow_bare_repo_with_config() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow bare.git
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/bare.yaml",
        "grow",
        "bare",
    ])?;

    // Ensure the repository was created
    // tests/tmp/grow-bare-repo-config/bare
    let bare_repo = fixture.path("bare");
    let repo = std::path::PathBuf::from(&bare_repo);
    assert!(repo.exists());

    // We cloned with --no-single-branch so "default" and "dev" must exist.
    assert_ref(&bare_repo, "default");
    assert_ref(&bare_repo, "dev");

    // The repository must be bare.
    let cmd = ["git", "config", "core.bare"];
    let output = assert_cmd_capture(&cmd, &bare_repo);
    assert_eq!(output, String::from("true"));

    Ok(())
}

/// `garden grow` sets up remotes
#[test]
#[named]
fn grow_remotes() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
    ])?;

    // remote.origin.url is a read-only https:// URL
    let worktree = fixture.path("example/tree/repo");
    let cmd = ["git", "config", "remote.origin.url"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert!(
        output.ends_with("/repos/example.git"),
        "{} does not end with {}",
        output,
        "/repos/example.git"
    );

    // remote.publish.url is a ssh push URL
    let cmd = ["git", "config", "remote.publish.url"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("git@github.com:user/example.git", output);

    Ok(())
}

/// `garden grow` creates symlinks
#[test]
#[named]
fn grow_symlinks() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree examples/link
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
        "link",
        "example/link",
    ])?;

    let repo = std::path::PathBuf::from(fixture.path("example/tree/repo/.git"));
    assert!(repo.exists());

    // tests/tmp/symlinks/link is a symlink pointing to example/tree/repo
    let link_str = fixture.path("link");
    let link = std::path::PathBuf::from(&link_str);
    assert!(link.exists(), "{} does not exist", link_str);
    assert!(link.read_link().is_ok());

    let target = link.read_link().unwrap();
    assert_eq!("example/tree/repo", target.to_string_lossy());

    // tests/tmp/symlinks/example/link is a symlink pointing to tree/repo
    let link_str = fixture.path("example/link");
    let link = std::path::PathBuf::from(&link_str);
    assert!(link.exists(), "{} does not exist", link_str);
    assert!(link.read_link().is_ok());

    let target = link.read_link().unwrap();
    assert_eq!("tree/repo", target.to_string_lossy());

    Ok(())
}

/// `garden grow` sets up git config settings
#[test]
#[named]
fn grow_gitconfig() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
    ])?;

    // remote.origin.annex-ignore is true
    let worktree = fixture.path("example/tree/repo");
    let cmd = ["git", "config", "remote.origin.annex-ignore"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("true", output);

    // user.name is "A U Thor"
    let cmd = ["git", "config", "user.name"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("A U Thor", output);

    // user.email is "author@example.com"
    let cmd = ["git", "config", "user.email"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("author@example.com", output);

    Ok(())
}

/// This creates a worktree
#[test]
#[named]
fn grow_worktree_and_parent() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow dev
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/worktree.yaml",
        "grow",
        "dev",
    ])?;

    // Ensure the repository was created
    let worktree_default = fixture.worktree("default");
    let worktree_dev = fixture.worktree("dev");

    assert_ref(&worktree_default, "default");
    assert_ref(&worktree_dev, "dev");

    // Ensure that the "echo" command is available from the child worktree.
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/worktree.yaml",
        "echo",
        "dev",
        "--",
        "hello",
    ]);
    // The "echo" command is: echo ${TREE_NAME} "$@"
    assert_eq!("dev hello", output);

    // Ensure that the "echo" command is available from the parent worktree.
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/worktree.yaml",
        "echo",
        "default",
        "--",
        "hello",
    ]);
    // The "echo" command is: echo ${TREE_NAME} "$@"
    assert_eq!("default hello", output);

    Ok(())
}

/// `garden plant` adds an empty repository
#[test]
#[named]
fn plant_empty_repo() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden plant in test/tmp/plant_empty_repo
    exec_garden(&["--chdir", &fixture.root(), "init"])?;

    // Empty garden.yaml should be created
    fixture.path("garden.yaml");

    // Create tests/tmp/plant_empty_repo/repo{1,2}
    let cmd = ["git", "init", "--quiet", "repo1"];
    assert_cmd(&cmd, &fixture.root());

    let cmd = ["git", "init", "--quiet", "repo2"];
    assert_cmd(&cmd, &fixture.root());

    // repo1 has two remotes: "origin" and "remote-1".
    // git remote add origin repo-1-url
    let cmd = ["git", "remote", "add", "origin", "repo-1-url"];
    let worktree_repo1 = fixture.worktree("repo1");
    assert_cmd(&cmd, &worktree_repo1);

    // git remote add remote-1 remote-1-url
    let cmd = ["git", "remote", "add", "remote-1", "remote-1-url"];
    assert_cmd(&cmd, &worktree_repo1);

    // garden plant repo1
    exec_garden(&["--chdir", &fixture.root(), "plant", "repo1"])?;

    let path = Some(std::path::PathBuf::from(fixture.path("garden.yaml")));

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
    let worktree_repo2 = fixture.worktree("repo2");
    let cmd = ["git", "remote", "add", "remote-1", "remote-1-url"];
    assert_cmd(&cmd, &worktree_repo2);

    // git remote add remote-2 remote-2-url
    let cmd = ["git", "remote", "add", "remote-2", "remote-2-url"];
    assert_cmd(&cmd, &worktree_repo2);

    // garden add repo2
    exec_garden(&["--chdir", &fixture.root(), "plant", "repo2"])?;

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
    let cmd = ["git", "config", "remote.origin.url", "repo-1-new-url"];
    assert_cmd(&cmd, &worktree_repo1);

    // Update repo2's remote-2 url to remote-2-new-url.
    // git config remote.remote-2.url remote-2-new-url
    let cmd = ["git", "config", "remote.remote-2.url", "remote-2-new-url"];
    assert_cmd(&cmd, &worktree_repo2);

    // garden plant repo1 repo2
    exec_garden(&["--chdir", &fixture.root(), "plant", "repo1", "repo2"])?;

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

    Ok(())
}

/// `garden plant` detects bare repositories.
#[test]
#[named]
fn plant_bare_repo() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // Create an empty garden.yaml using "garden init".
    exec_garden(&["--chdir", &fixture.root(), "init"])?;
    let garden_yaml = fixture.path("garden.yaml");

    let cmd = ["git", "init", "--quiet", "--bare", "repo.git"]; // Create repo.git
    assert_cmd(&cmd, &fixture.root());

    // garden plant repo.git
    exec_garden(&["--chdir", &fixture.root(), "plant", "repo.git"])?;

    // Load the configuration and assert that the remotes are configured.
    let path = Some(std::path::PathBuf::from(&garden_yaml));
    let cfg = garden::config::new(&path, "", 0, None)?;
    assert_eq!(1, cfg.trees.len());
    assert_eq!("repo.git", cfg.trees[0].get_name());

    // The generated config must have "bare: true" configured.
    assert!(cfg.trees[0].is_bare_repository);

    Ok(())
}

/// `garden eval` evaluates ${GARDEN_CONFIG_DIR}
#[test]
#[named]
fn eval_garden_config_dir() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden eval ${GARDEN_CONFIG_DIR}
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${GARDEN_CONFIG_DIR}",
    ]);
    assert!(
        output.ends_with("/tests/data"),
        "{} does not end with /tests/data",
        output
    );

    Ok(())
}

/// Test eval behavior around the "--root" option
#[test]
#[named]
fn eval_root_with_root() {
    // garden eval ${GARDEN_ROOT}
    let output = garden_capture(&[
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}

/// Test eval ${GARDEN_CONFIG_DIR} behavior with both "--root" and "--chdir"
#[test]
fn eval_config_dir_with_chdir_and_root() {
    let output = garden_capture(&[
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_CONFIG_DIR}",
    ]);
    assert!(output.ends_with("/tests/data"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}

/// Test pwd with both "--root" and "--chdir"
#[test]
fn eval_exec_pwd_with_root_and_chdir() {
    let output = garden_capture(&[
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "$ pwd",
    ]);
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}

/// Test ${GARDEN_ROOT} with both "--root" and "--chdir"
#[test]
fn eval_root_with_root_and_chdir() {
    let output = garden_capture(&[
        "--chdir",
        "tests/tmp",
        "--config",
        "tests/data/garden.yaml",
        "--root",
        "tests/tmp",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    assert!(output.ends_with("/tests/tmp"));

    let path = std::path::PathBuf::from(&output);
    assert!(path.exists());
    assert!(path.is_absolute());
}

/// Test dash-dash arguments in custom commands via "garden cmd ..."
#[test]
fn cmd_dash_dash_arguments() {
    let output = garden_capture(&[
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
    ]);
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
    let output = garden_capture(&[
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
    ]);
    // `. .` was used to operate on the tree twice.
    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(format!("{}\n{}", msg, msg), output);
}

/// Test "." default for custom "garden <command>" with no arguments
#[test]
fn cmd_dot_default_no_args() {
    let output = garden_capture(&["--quiet", "--chdir", "tests/data", "echo-dir"]);
    assert_eq!("data", output);
}

/// Test "." default for "garden <command>" with no arguments and echo
#[test]
fn cmd_dot_default_no_args_echo() {
    let output = garden_capture(&["--quiet", "--chdir", "tests/data", "echo-args"]);
    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(msg, output);
}

/// Test "." default for "garden <command>" with double-dash
#[test]
fn cmd_dot_default_double_dash() {
    let output = garden_capture(&["--quiet", "--chdir", "tests/data", "echo-args", "--"]);
    let msg = "garden\narguments -- a b c -- -- x y z";
    assert_eq!(msg, output);
}

/// Test "." default for "garden <command>" with extra arguments
#[test]
fn cmd_dot_default_double_dash_args() {
    let output = garden_capture(&[
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
    ]);
    let msg = "garden\narguments -- a b c -- d e f -- g h i -- x y z";
    assert_eq!(msg, output);
}
