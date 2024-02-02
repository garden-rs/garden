pub mod common;
use common::{
    assert_cmd, assert_cmd_capture, assert_ref, assert_ref_missing, exec_garden, garden_capture,
    BareRepoFixture,
};

use garden::git;
use garden::model;

use anyhow::Result;
use function_name::named;

/// `garden init` adds the current repository
#[test]
#[named]
fn init_adds_repository() -> Result<()> {
    let fixture = common::BareRepoFixture::new(function_name!());
    // garden init in test/tmp/init_adds_repository
    exec_garden(&["--chdir", &fixture.root(), "init"])?;
    // Non-empty garden.yaml should be created
    fixture.path("garden.yaml");
    let pathbuf = fixture.pathbuf("garden.yaml");
    let app_context = garden::model::ApplicationContext::from_path(pathbuf)?;
    let cfg = app_context.get_root_config();
    assert_eq!(1, cfg.trees.len());
    assert_eq!(function_name!(), cfg.trees[0].get_name());

    Ok(())
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
    let lines = output.split('\n');
    assert_eq!(
        lines.count(),
        1,
        "git rev-list HEAD outputs only one commit "
    );

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
    let lines = output.split('\n');
    assert_eq!(
        lines.count(),
        1,
        "git rev-list HEAD outputs only one commit "
    );

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
    let lines = output.split('\n').collect::<Vec<&str>>();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "default");

    // The "dev" repository must have a branch called "dev" checked-out.
    let cmd = ["git", "symbolic-ref", "--short", "HEAD"];
    let output = assert_cmd_capture(&cmd, &worktree_dev);
    let lines = output.split('\n').collect::<Vec<&str>>();
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
    assert_eq!(output.as_str(), "true");

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
    assert_eq!(output.as_str(), "true");

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

    // remote.publish.tagopt is --no-tags
    let cmd = ["git", "config", "remote.publish.tagopt"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("--no-tags", output);

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

    let repo = fixture.pathbuf("example/tree/repo/.git");
    assert!(repo.exists());

    // tests/tmp/symlinks/link is a symlink pointing to example/tree/repo
    let link = fixture.pathbuf("link");
    assert!(link.exists(), "{link:?} must exist");
    assert!(link.read_link().is_ok());

    let target = link.read_link().unwrap();
    assert_eq!("example/tree/repo", target.to_string_lossy());

    // tests/tmp/symlinks/example/link is a symlink pointing to tree/repo
    let link = fixture.pathbuf("example/link");
    assert!(link.exists(), "{link:?} does not exist");
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

/// `garden grow` sets up git config settings
#[test]
#[named]
fn grow_gitconfig_append_value() -> Result<()> {
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

    // remote.origin.pushurl is configured with two values.
    let worktree = fixture.path("example/tree/repo");
    let cmd = ["git", "config", "--get-all", "remote.origin.pushurl"];
    let output = assert_cmd_capture(&cmd, &worktree);
    assert_eq!("url1\nurl2", output);

    Ok(())
}

/// `garden grow` sets up remote tracking branches configured in trees.<tree>.branches.<name>.
#[test]
#[named]
fn grow_branches_for_clone() -> Result<()> {
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
        "local",
    ])?;

    let local_repo = fixture.path("local");

    // The "local" branch must be checked out.
    let cmd = ["git", "symbolic-ref", "HEAD"];
    let output = assert_cmd_capture(&cmd, &local_repo);
    assert_eq!("refs/heads/local", output);

    // Ensure that both "local" and "dev" branches are created.
    let cmd_dev = ["git", "rev-parse", "local"];
    let cmd_local = ["git", "rev-parse", "dev"];
    let output_local = assert_cmd_capture(&cmd_local, &local_repo);
    let output_dev = assert_cmd_capture(&cmd_dev, &local_repo);
    assert_eq!(output_dev, output_local);

    // The upstream branches must be configured.
    let cmd = ["git", "config", "branch.dev.remote"];
    let output = assert_cmd_capture(&cmd, &local_repo);
    assert_eq!("origin", output);

    let cmd = ["git", "config", "branch.local.remote"];
    let output = assert_cmd_capture(&cmd, &local_repo);
    assert_eq!("origin", output);

    let cmd = ["git", "config", "branch.dev.merge"];
    let output = assert_cmd_capture(&cmd, &local_repo);
    assert_eq!("refs/heads/dev", output);

    let cmd = ["git", "config", "branch.local.merge"];
    let output = assert_cmd_capture(&cmd, &local_repo);
    assert_eq!("refs/heads/default", output);

    Ok(())
}

/// Create a child worktrees using "git worktree".
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

/// `garden grow` uses the configured default remote when just "url" is configured.
#[test]
#[named]
fn grow_default_remote_name() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow example/default-remote-name
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/default-remote-name",
    ])?;
    // Ensure that both "main" and "custom/main" branches are created.
    let repo = fixture.worktree("example/tree/default-remote");
    let cmd_main = ["git", "rev-parse", "default"];
    let cmd_custom_main = ["git", "rev-parse", "custom/default"];
    let main_commit_id = assert_cmd_capture(&cmd_main, &repo);
    let custom_main_commit_id = assert_cmd_capture(&cmd_custom_main, &repo);
    assert_eq!(main_commit_id, custom_main_commit_id);
    // The "checkout.defaultRemoteName" configuration must be setup.
    let cmd = ["git", "config", "checkout.defaultRemoteName"];
    let output = assert_cmd_capture(&cmd, &repo);
    assert_eq!("custom", output);
    // The "remote.origin.url" configuration must be setup.
    let cmd = ["git", "config", "remote.origin.url"];
    let output = assert_cmd_capture(&cmd, &repo);
    assert_eq!("git://git.example.org/example.git", output);

    Ok(())
}

/// `garden grow` uses the configured default named remote.
#[test]
#[named]
fn grow_default_remote_url() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow example/default-remote-url
    exec_garden(&[
        "--verbose",
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/default-remote-url",
    ])?;
    // Ensure that both "main" and "custom/main" branches are created.
    let repo = fixture.worktree("example/tree/default-remote");
    let cmd_main = ["git", "rev-parse", "default"];
    let cmd_custom_main = ["git", "rev-parse", "custom/default"];
    let main_commit_id = assert_cmd_capture(&cmd_main, &repo);
    let custom_main_commit_id = assert_cmd_capture(&cmd_custom_main, &repo);
    assert_eq!(main_commit_id, custom_main_commit_id);
    // The "checkout.defaultRemoteName" configuration must be setup.
    let cmd = ["git", "config", "checkout.defaultRemoteName"];
    let output = assert_cmd_capture(&cmd, &repo);
    assert_eq!("custom", output);

    Ok(())
}

/// `garden eval` evaluates ${GARDEN_CONFIG_DIR}
#[test]
#[named]
fn eval_garden_config_dir() {
    let fixture = BareRepoFixture::new(function_name!());
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${GARDEN_CONFIG_DIR}",
    ]);
    let expect = "/tests/data";
    assert!(
        output.ends_with(expect),
        "GARDEN_ROOT ({output}) does not end with {expect}"
    );
}

/// `garden eval` evaluates ${GARDEN_ROOT}
#[test]
#[named]
fn eval_garden_root() {
    let fixture = BareRepoFixture::new(function_name!());
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    let expect = "/tests/tmp/eval_garden_root";
    assert!(
        output.ends_with(expect),
        "GARDEN_ROOT ({output}) does not end with {expect}"
    );
    // This garden file does not configure `garden.root`.
    // The config directory (GARDEN_CONFIG_DIR) is used as the GARDEN_ROOT.
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/default.yaml",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    let expect = "/tests/data";
    assert!(
        output.ends_with(expect),
        "GARDEN_ROOT ({output}) does not end with {expect}"
    );
}

/// `garden eval` evaluates ${GARDEN_CONFIG_DIR}
#[test]
#[named]
fn eval_override_variables() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden eval ${tree_variable} current
    let output = garden_capture(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "--define",
        "tree_value=test",
        "eval",
        "${tree_value}",
        "current",
    ]);
    assert_eq!(output, "test");

    Ok(())
}

/// `garden eval` evaluates ${GARDEN_ROOT} to the same directory as the
/// garden config directory when `garden.root` is unspecified.
#[test]
fn eval_default_config_dir() {
    // garden eval ${tree_variable} current
    let output = garden_capture(&[
        "--config",
        "tests/data/config/garden.yaml",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    assert!(output.ends_with("/tests/data/config"));

    let output = garden_capture(&[
        "--config",
        "tests/data/config/garden.yaml",
        "exec",
        "current",
        "pwd",
    ]);
    assert!(output.ends_with("/tests/data/config"));
}

/// `garden eval` evaluates variables from "environment" blocks.
#[test]
fn eval_environment_blocks() {
    // garden --chdir tests/data eval '${GARDEN_ENV_VALUE}' trees/prebuilt
    let expect = "trees/prebuilt/env/value";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data",
        "eval",
        "${GARDEN_ENV_VALUE}",
        "trees/prebuilt",
    ]);
    assert_eq!(expect, actual);
    // garden --chdir tests/data eval --env '${GARDEN_ENV_VALUE}' graft::prebuilt
    let expect = "graft/grafted-env/env/value";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data",
        "eval",
        "${GARDEN_ENV_VALUE}",
        "graft::grafted-env",
    ]);
    assert_eq!(expect, actual);
    // garden --chdir tests/data eval --env '${GARDEN_ENV_VALUE}' trees/prebuilt garden/env
    let expect = "garden/env:graft/grafted-env/env/value:trees/prebuilt/env/value";
    let actual = garden_capture(&[
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${GARDEN_ENV_VALUE}",
        "trees/prebuilt",
        "garden/env",
    ]);
    assert_eq!(expect, actual);
    // garden --chdir tests/data eval --env '${GARDEN_ENV_VALUE}' graft::prebuilt garden/env
    let expect = "garden/env:graft/grafted-env/env/value:trees/prebuilt/env/value";
    let actual = garden_capture(&[
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${GARDEN_ENV_VALUE}",
        "graft::grafted-env",
        "garden/env",
    ]);
    assert_eq!(expect, actual);
}

/// `garden::git::worktree_details(path)` returns a struct with branches and a
/// GitTreeType (Tree, Bare, Parent, Worktree) for this worktree.
#[test]
#[named]
fn git_worktree_details() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());

    // repos/example.git is a bare repository.
    let details = git::worktree_details(&fixture.pathbuf("repos/example.git"))?;
    assert_eq!(details.branch, ""); // Bare repository has no branch.
    assert_eq!(details.tree_type, model::GitTreeType::Bare);

    // Create a plain git worktree called "tree" with a branch called "branch".
    let cmd = ["git", "init", "--quiet", "tree"];
    assert_cmd(&cmd, &fixture.root());

    let cmd = ["git", "symbolic-ref", "HEAD", "refs/heads/branch"];
    assert_cmd(&cmd, &fixture.path("tree"));

    let details = git::worktree_details(&fixture.pathbuf("tree"))?;
    assert_eq!(details.branch, "branch");
    assert_eq!(details.tree_type, model::GitTreeType::Tree);

    // Create a parent worktree called "parent" branched off of "default".
    let cmd = ["git", "clone", "--quiet", "repos/example.git", "parent"];
    assert_cmd(&cmd, &fixture.root());

    // The initial query will be a Tree because there are no child worktrees.
    let details = git::worktree_details(&fixture.pathbuf("parent"))?;
    assert_eq!(details.branch, "default");
    assert_eq!(details.tree_type, model::GitTreeType::Tree);

    // Create a child worktree called "child" branched off of "dev".
    let cmd = [
        "git",
        "worktree",
        "add",
        "--track",
        "-B",
        "dev",
        "../child",
        "origin/dev",
    ];
    assert_cmd(&cmd, &fixture.path("parent"));

    // The "parent" repository is a GitTreeType::Parent.
    let details = git::worktree_details(&fixture.pathbuf("parent"))?;
    assert_eq!(details.branch, "default");
    assert_eq!(details.tree_type, model::GitTreeType::Parent);

    // The "child" repository is a GitTreeType::Worktree(parent_path).
    let parent_path = garden::path::abspath(&fixture.pathbuf("parent"));
    let details = git::worktree_details(&fixture.pathbuf("child"))?;
    assert_eq!(details.branch, "dev");
    assert_eq!(details.tree_type, model::GitTreeType::Worktree(parent_path));

    Ok(())
}

/// Test eval behavior around the "--root" option
#[test]
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

/// Test evaluating a namespaced graft::variable.
#[test]
fn eval_grafted_variable_with_namespace() {
    let output = garden_capture(&[
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${graft::variable}",
    ]);
    assert_eq!(output, "graft value");
}

/// Test evaluating a variable at global scope that references a graft::variable.
#[test]
fn eval_grafted_variable_at_global_scope() {
    let output = garden_capture(&["--config", "tests/data/garden.yaml", "eval", "${variable}"]);
    assert_eq!(output, "global graft value");
}

/// Test evaluating a variable at tree scope that references a graft::variable.
#[test]
fn eval_graft_variable_at_tree_scope() {
    let output = garden_capture(&[
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${variable}",
        "trees/prebuilt",
    ]);
    assert_eq!(output, "prebuilt graft value");
}

/// `garden grow` creates symlinks
#[test]
#[named]
fn git_branches() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    let root = fixture.root();
    fixture.assert_worktree(&root);

    let cmd = ["git", "branch", "abc"];
    assert_cmd(&cmd, &root);

    let cmd = ["git", "branch", "xyz"];
    assert_cmd(&cmd, &root);

    let branches = git::branches(&fixture.root_pathbuf());
    assert_eq!(branches.len(), 3); // Default branch + abc + xyz
    assert!(branches.contains(&"abc".to_string()));
    assert!(branches.contains(&"xyz".to_string()));

    Ok(())
}

/// `garden eval` evaluates builtin from the perspective of the graft.
#[test]
fn eval_grafted_builtin_variables() {
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "eval",
        "${GARDEN_CONFIG_DIR}",
        "graft::prebuilt",
    ]);
    let expect = "/tests/data/grafts";
    assert!(
        output.ends_with(expect),
        "Grafted GARDEN_CONFIG_DIR ({output}) must use the grafted {expect} path"
    );

    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "eval",
        "${TREE_PATH}",
        "graft::prebuilt",
    ]);
    let expect = "/tests/data/trees/prebuilt";
    assert!(
        output.ends_with(expect),
        "Grafted TREE_PATH ({output}) must be in {expect}"
    );

    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "eval",
        "${TREE_PATH}",
        "graft-no-root::tree",
    ]);
    let expect = "/tests/data/grafts/trees/tree";
    assert!(
        output.ends_with(expect),
        "Grafted TREE_PATH ({output}) must be in {expect}"
    );

    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "eval",
        "${GARDEN_ROOT}",
        "graft::prebuilt",
    ]);
    let expect = "/tests/data";
    assert!(
        output.ends_with(expect),
        "Grafted GARDEN_ROOT ({output}) must use {expect} from the current directory"
    );
    // This time we --chdir to tests/ instead and see it reflected in GARDEN_ROOT.
    let output = garden_capture(&[
        "--chdir",
        "tests",
        "--config",
        "tests/data/garden.yaml",
        "eval",
        "${GARDEN_ROOT}",
        "graft::prebuilt",
    ]);
    let expect = "/tests";
    assert!(
        output.ends_with(expect),
        "Grafted GARDEN_ROOT ({output}) must use {expect} from the current directory"
    );
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
    assert_eq!(output, format!("{msg}\n{msg}"));
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
    assert_eq!(format!("{msg}\n{msg}"), output);
}

/// Test the creation of an implicit tree for the current directory.
#[test]
fn cmd_default_tree() {
    let expect = "hello world";
    let actual = garden_capture(&[
        "--config",
        "tests/data/commands/garden.yaml",
        "echo",
        "--",
        "hello",
        "world",
    ]);
    assert_eq!(expect, actual);

    let expect = "/tests/data/commands";
    let actual = garden_capture(&["--config", "tests/data/commands/garden.yaml", "pwd"]);
    assert!(
        actual.ends_with(expect),
        "pwd output ({actual}) must be in {expect}"
    );

    let expect = "/tests/data/commands";
    let actual = garden_capture(&[
        "--config",
        "tests/data/commands/garden.yaml",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    assert!(
        actual.ends_with(expect),
        "GARDEN_ROOT ({actual}) must be in {expect}"
    );

    let expect = "/tests/data/commands";
    let actual = garden_capture(&[
        "--config",
        "tests/data/commands/garden.yaml",
        "eval",
        "${GARDEN_CONFIG_DIR}",
    ]);
    assert!(
        actual.ends_with(expect),
        "GARDEN_CONFIG_DIR ({actual}) must be in {expect}"
    );

    let expect = "/tests/data/commands";
    let actual = garden_capture(&[
        "--config",
        "tests/data/commands/garden.yaml",
        "eval",
        "${TREE_PATH}",
        ".",
    ]);
    assert!(
        actual.ends_with(expect),
        "TREE_PATH ({actual}) must be in {expect}"
    );

    let expect = ".";
    let actual = garden_capture(&[
        "--config",
        "tests/data/commands/garden.yaml",
        "eval",
        "${TREE_NAME}",
        ".",
    ]);
    assert_eq!(expect, actual);
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

/// Test "garden cmd --breadth-first ..."
/// Test "garden cmd ..."
#[test]
fn cmd_breadth_first_and_depth_first() {
    // Commands are run in breadth-first order.
    // Each command is run in each tree before proceeding to the next command.
    let expect = "tree1\ntree2\nx1\nx2";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        "--breadth-first",
        "trees",
        "tree-name",
        "tree-var",
    ]);
    assert_eq!(expect, actual);

    // Commands are run in depth-first order.
    // All commands are run in each tree before proceeding to the next tree.
    let expect = "tree1\nx1\ntree2\nx2";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        "trees",
        "tree-name",
        "tree-var",
    ]);
    assert_eq!(expect, actual);
}

/// Test -n / --no-errexit and the shell "-e" behavior.
#[test]
fn cmd_no_errexit() {
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        ".",
        "error-command",
    ]);
    assert_eq!(output, "ok");

    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        "--no-errexit",
        ".",
        "error-command",
    ]);
    assert_eq!(output, "ok\nafter error");
}

#[test]
fn cmd_no_errexit_for_command_lists() {
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        ".",
        "error-command-list",
    ]);
    assert_eq!(output, "ok");

    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        "--no-errexit",
        ".",
        "error-command-list",
    ]);
    assert_eq!(output, "ok\nafter error");
}

/// Test the interaction of --keep-going, --no-errexit and command lists.
#[test]
fn cmd_keep_going_and_no_errexit() {
    // exit-on-error: true, keep-going: false, command: str
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "error-command",
        "tree1",
        "tree2",
    ]);
    assert_eq!(output, "ok");

    // exit-on-error: false, keep-going: false, command: str
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "error-command",
        "--no-errexit",
        "tree1",
        "tree2",
    ]);
    assert_eq!(output, "ok\nafter error");

    // exit-on-error: true, keep-going: true, command: str
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "error-command",
        "--keep-going",
        "tree1",
        "tree2",
    ]);
    assert_eq!(output, "ok\nok");

    // exit-on-error: false, keep-going: true, command: str
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "error-command",
        "--keep-going",
        "--no-errexit",
        "tree1",
        "tree2",
    ]);
    assert_eq!(output, "ok\nafter error\nok\nafter error");

    // Same as above but with error-command-list instead of error-command.
    // exit-on-error: true, keep-going: false, command: list
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "error-command-list",
        "tree1",
        "tree2",
    ]);
    assert_eq!(output, "ok");

    // exit-on-error: false, keep-going: false, command: list
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "error-command-list",
        "--no-errexit",
        "tree1",
        "tree2",
    ]);
    assert_eq!(output, "ok\nafter error");

    // exit-on-error: true, keep-going: true, command: list
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "error-command-list",
        "--keep-going",
        "tree1",
        "tree2",
    ]);
    assert_eq!(output, "ok\nok");

    // exit-on-error: false, keep-going: true, command: list
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "error-command-list",
        "--keep-going",
        "--no-errexit",
        "tree1",
        "tree2",
    ]);
    assert_eq!(output, "ok\nafter error\nok\nafter error");

    // exit-on-error: true, keep-going: false, command: multi
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        "tree*",
        "error-command",
        "error-command-list",
    ]);
    assert_eq!(output, "ok");

    // exit-on-error: false, keep-going: false, command: multi
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        "--no-errexit",
        "tree*",
        "error-command",
        "error-command-list",
    ]);
    assert_eq!(output, "ok\nafter error");

    // exit-on-error: true, keep-going: true, command: multi
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        "--keep-going",
        "tree*",
        "error-command",
        "error-command-list",
    ]);
    assert_eq!(output, "ok\nok\nok\nok");
}

/// Test pre and post-commands
#[test]
fn cmd_pre_and_post_commands() {
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        ".",
        "echo-pre-and-post",
    ]);
    let output_len = output.lines().count();
    assert_eq!(output_len, 4);
    assert_eq!(output, "pre\ncmd\ndata\npost");

    let output = garden_capture(&["--chdir", "tests/data", "--quiet", "echo-pre-and-post"]);
    let output_len = output.lines().count();
    assert_eq!(output_len, 4);
    assert_eq!(output, "pre\ncmd\ndata\npost");
}

/// Test pre and post nested commands
#[test]
fn cmd_pre_and_post_nested_commands() {
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        ".",
        "echo-pre-and-post-nested",
    ]);
    let output_len = output.lines().count();
    assert_eq!(output_len, 6);
    assert_eq!(output, "pre\ncmd\ndata\npost\nnested\nfini");

    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "echo-pre-and-post-nested",
    ]);
    let output_len = output.lines().count();
    assert_eq!(output_len, 6);
    assert_eq!(output, "pre\ncmd\ndata\npost\nnested\nfini");
}

/// Test the use of graft:: tree references in groups.
#[test]
fn cmd_exec_group_with_grafted_trees() {
    // grafted-group is a group with graft::prebuilt.
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "exec",
        "grafted-group",
        "pwd",
    ]);
    let output_len = output.lines().count();
    assert_eq!(output_len, 2);

    for line in output.lines() {
        assert!(line.ends_with("/tests/data/trees/prebuilt"));
    }
}

/// Test the use of graft:: tree references in gardens.
#[test]
fn cmd_exec_garden_with_grafted_trees() {
    // grafted-graden is a garden with graft::prebuilt.
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "exec",
        "grafted-garden",
        "pwd",
    ]);
    let output_len = output.lines().count();
    assert_eq!(output_len, 2);

    for line in output.lines() {
        assert!(line.ends_with("/tests/data/trees/prebuilt"));
    }
}

/// Test the use of graft:: tree references when querying for groups.
#[test]
fn cmd_exec_grafted_group() {
    // prebuilt-group has two trees pointing to the same prebuilt path.
    let output = garden_capture(&[
        "--quiet",
        "--chdir",
        "tests/data",
        "exec",
        "graft::prebuilt-group",
        "pwd",
    ]);
    let output_len = output.lines().count();
    assert_eq!(output_len, 2);

    for line in output.lines() {
        assert!(line.ends_with("/tests/data/trees/prebuilt"));
    }
}

/// Test garden file discovery when run from a subdirectory.
#[test]
fn cmd_garden_discovery() {
    let expect = "/tests/data";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data/trees/prebuilt",
        "--config",
        "default.yaml",
        "eval",
        "${GARDEN_CONFIG_DIR}",
    ]);
    assert!(
        actual.ends_with(expect),
        "GARDEN_CONFIG_DIR ({actual}) should be in {expect}"
    );

    let expect = "/tests/data";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data/trees/prebuilt",
        "--config",
        "default.yaml",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    assert!(
        actual.ends_with(expect),
        "GARDEN_ROOT ({actual}) should be in {expect}"
    );

    let expect = "/tests/data/trees/prebuilt";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data/trees/prebuilt",
        "eval",
        "${GARDEN_ROOT}",
    ]);
    assert!(
        actual.ends_with(expect),
        "GARDEN_ROOT ({actual}) should be in {expect}"
    );

    let expect = "/tests/data/trees/prebuilt";
    let actual = garden_capture(&[
        "--root",
        "${GARDEN_CONFIG_DIR}",
        "--chdir",
        "tests/data/trees/prebuilt",
        "eval",
        "${TREE_PATH}",
        ".",
    ]);
    assert!(
        actual.ends_with(expect),
        "TREE_PATH ({actual}) should be in {expect}"
    );

    let expect = "/tests/data/trees/prebuilt";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data/trees/prebuilt",
        "--config",
        "default.yaml",
        "eval",
        "${TREE_PATH}",
        ".",
    ]);
    assert!(
        actual.ends_with(expect),
        "TREE_PATH ({actual}) should be in {expect}"
    );

    let expect = "prebuilt";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data/trees/prebuilt",
        "--config",
        "default.yaml",
        "eval",
        "${TREE_NAME}",
        ".",
    ]);
    assert_eq!(expect, actual);

    let expect = "/tests/data";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data/trees",
        "--config",
        "default.yaml",
        "eval",
        "${TREE_PATH}",
        ".",
    ]);
    assert!(
        actual.ends_with(expect),
        "TREE_PATH ({actual}) should be in {expect}"
    );

    let expect = "current";
    let actual = garden_capture(&[
        "--chdir",
        "tests/data/trees",
        "--config",
        "default.yaml",
        "eval",
        "${TREE_NAME}",
        ".",
    ]);
    assert_eq!(expect, actual);
}

/// Test tree filtering using custom commands
#[test]
fn cmd_custom_filtered_trees() {
    let expect = "current\nprebuilt";
    let actual = garden_capture(&["--config", "tests/data/default.yaml", "name", "*"]);
    assert_eq!(expect, actual);

    let expect = "current";
    let actual = garden_capture(&[
        "--config",
        "tests/data/default.yaml",
        "name",
        "--trees",
        "cur*",
        "*",
    ]);
    assert_eq!(expect, actual);

    let expect = "prebuilt";
    let actual = garden_capture(&[
        "--config",
        "tests/data/default.yaml",
        "name",
        "--trees",
        "pre*",
        "*",
    ]);
    assert_eq!(expect, actual);

    let expect = "current\nprebuilt";
    let actual = garden_capture(&["--config", "tests/data/default.yaml", "cmd", "*", "name"]);
    assert_eq!(expect, actual);

    let expect = "current";
    let actual = garden_capture(&[
        "--config",
        "tests/data/default.yaml",
        "cmd",
        "--trees",
        "cur*",
        "*",
        "name",
    ]);
    assert_eq!(expect, actual);

    let expect = "prebuilt";
    let actual = garden_capture(&[
        "--config",
        "tests/data/default.yaml",
        "cmd",
        "--trees",
        "pre*",
        "*",
        "name",
    ]);
    assert_eq!(expect, actual);
}

/// Test tree filtering using exec.
#[test]
fn cmd_exec_filtered_trees() {
    let actual = garden_capture(&["--config", "tests/data/default.yaml", "exec", "*", "pwd"]);
    assert_eq!(actual.lines().count(), 2);

    let expect = "/tests/data";
    let actual = garden_capture(&[
        "--config",
        "tests/data/default.yaml",
        "exec",
        "--trees",
        "cur*",
        "*",
        "pwd",
    ]);
    assert_eq!(actual.lines().count(), 1);
    assert!(actual.ends_with(expect));

    let expect = "/tests/data/trees/prebuilt";
    let actual = garden_capture(&[
        "--config",
        "tests/data/default.yaml",
        "exec",
        "--trees",
        "pre*",
        "*",
        "pwd",
    ]);
    assert_eq!(actual.lines().count(), 1);
    assert!(actual.ends_with(expect));
}

/// Test the use of $shell variables in commands.
/// $shell variables are not expanded by garden.
/// ${garden} variables are expanded.
#[test]
fn cmd_shell_variables() {
    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        ".",
        "echo-variable",
        "--",
        "test",
        "value",
    ]);
    // Repeated command names were used to operate on the tree twice.
    assert_eq!(output, "garden test shell value expr");

    let output = garden_capture(&[
        "--chdir",
        "tests/data",
        "--quiet",
        "cmd",
        ".",
        "echo-escaped",
        "--",
        "test",
        "value",
    ]);
    // Repeated command names were used to operate on the tree twice.
    assert_eq!(output, "test array value");
}

/// "garden prune" prunes specific depths
#[test]
#[named]
fn cmd_prune_depth() -> Result<()> {
    let fixture = BareRepoFixture::new(function_name!());
    // garden grow examples/tree creates "example/tree".
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "grow",
        "example/tree",
    ])?;
    let example_path = fixture.pathbuf("example");
    let mut example_tree_path = example_path.clone();
    example_tree_path.push("tree");
    assert!(example_tree_path.exists(), "example/tree must exist");

    // Create example/unknown.
    let cmd = ["git", "init", "--quiet", "example/unknown"];
    assert_cmd(&cmd, &fixture.root());

    // Prune the example/ directory (dry-run mode).
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "prune",
        "--no-prompt",
        "example",
    ])?;

    let mut example_unknown_path = example_path;
    example_unknown_path.push("unknown");
    assert!(example_tree_path.exists(), "example/tree must exist");
    assert!(
        example_unknown_path.exists(),
        "example/unknown must exist (dry-run)"
    );

    // Prune the example/ directory.
    // This is the same "garden prune" command as above plus "--rm" to enable deletion.
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "prune",
        "--no-prompt",
        "--rm",
        "example",
    ])?;

    assert!(example_tree_path.exists(), "example/tree must be retained");
    assert!(
        !example_unknown_path.exists(),
        "example/unknown must be removed"
    );

    // Create level0-unknown, level1/unknown, level1/level2/unknown, level1/level2/level3/unknown
    assert_cmd(
        &["git", "init", "--quiet", "level0-unknown"],
        &fixture.root(),
    );
    assert_cmd(
        &["git", "init", "--quiet", "level1/unknown"],
        &fixture.root(),
    );
    assert_cmd(
        &["git", "init", "--quiet", "level1/level2/unknown"],
        &fixture.root(),
    );
    assert_cmd(
        &["git", "init", "--quiet", "level1/level2/level3/unknown"],
        &fixture.root(),
    );

    let level0_unknown_path = fixture.pathbuf("level0-unknown");
    let level1_path = fixture.pathbuf("level1"); // level/
    let mut level2_path = level1_path.clone();
    level2_path.push("level2"); // level1/level2/
    let mut level3_path = level2_path.clone();
    level3_path.push("level3"); // level1/level2/level3/
    let mut level1_unknown_path = level1_path.clone();
    level1_unknown_path.push("unknown"); // level1/unknown
    let mut level2_unknown_path = level2_path.clone();
    level2_unknown_path.push("unknown"); // level1/level2/unknown
    let mut level3_unknown_path = level3_path.clone();
    level3_unknown_path.push("unknown"); // level1/level2/level3/unknown

    assert!(level1_path.exists());
    assert!(level2_path.exists());
    assert!(level3_path.exists());
    assert!(level0_unknown_path.exists());
    assert!(level1_unknown_path.exists());
    assert!(level2_unknown_path.exists());
    assert!(level3_unknown_path.exists());

    // Prune level 1 only.
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "prune",
        "--no-prompt",
        "--rm",
        "--exact-depth",
        "1",
    ])?;
    assert!(level1_path.exists());
    assert!(level2_path.exists());
    assert!(level3_path.exists());
    assert!(level0_unknown_path.exists());
    assert!(!level1_unknown_path.exists()); // Only level1/unknown should be removed.
    assert!(level2_unknown_path.exists());
    assert!(level3_unknown_path.exists());

    // Prune with at max-depth 0.
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "prune",
        "--no-prompt",
        "--rm",
        "--max-depth",
        "0",
    ])?;
    assert!(level1_path.exists());
    assert!(level2_path.exists());
    assert!(level3_path.exists());
    assert!(
        !level0_unknown_path.exists(),
        "level0-unknown must be removed"
    );
    // level1/unknown was removed from the previous "garden prune".
    assert!(level2_unknown_path.exists());
    assert!(level3_unknown_path.exists());

    // Prune with no limits with a bogus filter. Nothing should be removed.
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "prune",
        "--no-prompt",
        "--rm",
        "bogus-filter",
    ])?;
    // Nothing was removed.
    assert!(level1_path.exists());
    assert!(level2_path.exists());
    assert!(level3_path.exists());
    assert!(level2_unknown_path.exists());
    assert!(level3_unknown_path.exists());

    // Prune with min-depth 4. Nothing should be removed.
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "prune",
        "--no-prompt",
        "--rm",
        "--min-depth",
        "4",
    ])?;
    // Nothing was removed.
    assert!(level1_path.exists());
    assert!(level2_path.exists());
    assert!(level3_path.exists());
    assert!(level2_unknown_path.exists());
    assert!(level3_unknown_path.exists());

    // Prune with min-depth 3. level3 and below should be removed.
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "prune",
        "--no-prompt",
        "--rm",
        "--min-depth",
        "3",
    ])?;
    // level3 was removed.
    assert!(level1_path.exists());
    assert!(level2_path.exists());
    assert!(level2_unknown_path.exists());
    assert!(!level3_path.exists());
    assert!(!level3_unknown_path.exists());

    // Prune with no limits with a valid filter.
    exec_garden(&[
        "--verbose",
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/garden.yaml",
        "prune",
        "--no-prompt",
        "--rm",
        "level1",
    ])?;
    // level1 and below should be removed.
    assert!(!level1_path.exists());
    assert!(!level2_path.exists());
    assert!(!level1_unknown_path.exists());
    assert!(!level2_unknown_path.exists());

    Ok(())
}
