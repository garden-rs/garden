pub mod common;

use anyhow::Result;
use function_name::named;

/// `garden plant` adds an empty repository
#[test]
#[named]
fn plant_empty_repo() -> Result<()> {
    let fixture = common::BareRepoFixture::new(function_name!());
    // garden plant in test/tmp/plant_empty_repo
    common::exec_garden(&["--chdir", &fixture.root(), "init"])?;

    // Empty garden.yaml should be created
    fixture.path("garden.yaml");

    // Create tests/tmp/plant_empty_repo/repo{1,2}
    let cmd = ["git", "init", "--quiet", "repo1"];
    common::assert_cmd(&cmd, &fixture.root());

    let cmd = ["git", "init", "--quiet", "repo2"];
    common::assert_cmd(&cmd, &fixture.root());

    // repo1 has two remotes: "origin" and "remote-1".
    // git remote add origin repo-1-url
    let cmd = ["git", "remote", "add", "origin", "repo-1-url"];
    let worktree_repo1 = fixture.worktree("repo1");
    common::assert_cmd(&cmd, &worktree_repo1);

    // git remote add remote-1 remote-1-url
    let cmd = ["git", "remote", "add", "remote-1", "remote-1-url"];
    common::assert_cmd(&cmd, &worktree_repo1);

    // garden plant repo1
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "repo1"])?;

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
    common::assert_cmd(&cmd, &worktree_repo2);

    // git remote add remote-2 remote-2-url
    let cmd = ["git", "remote", "add", "remote-2", "remote-2-url"];
    common::assert_cmd(&cmd, &worktree_repo2);

    // garden add repo2
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "repo2"])?;

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
    common::assert_cmd(&cmd, &worktree_repo1);

    // Update repo2's remote-2 url to remote-2-new-url.
    // git config remote.remote-2.url remote-2-new-url
    let cmd = ["git", "config", "remote.remote-2.url", "remote-2-new-url"];
    common::assert_cmd(&cmd, &worktree_repo2);

    // garden plant repo1 repo2
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "repo1", "repo2"])?;

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
    let fixture = common::BareRepoFixture::new(function_name!());
    // Create an empty garden.yaml using "garden init".
    common::exec_garden(&["--chdir", &fixture.root(), "init"])?;
    let garden_yaml = fixture.path("garden.yaml");

    // garden plant repo.git
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "repos/example.git"])?;

    // Load the configuration and assert that the remotes are configured.
    let path = Some(std::path::PathBuf::from(&garden_yaml));
    let cfg = garden::config::new(&path, "", 0, None)?;
    assert_eq!(1, cfg.trees.len());
    assert_eq!("repos/example.git", cfg.trees[0].get_name());

    // The generated config must have "bare: true" configured.
    assert!(cfg.trees[0].is_bare_repository);

    Ok(())
}

/// `garden plant` detects "git worktree" repositories.
#[test]
#[named]
fn plant_git_worktree() -> Result<()> {
    let fixture = common::BareRepoFixture::new(function_name!());
    // Create an empty garden.yaml using "garden init".
    common::exec_garden(&["--chdir", &fixture.root(), "init"])?;

    // Create a parent worktree called "parent" branched off of "default".
    let cmd = ["git", "clone", "--quiet", "repos/example.git", "parent"];
    common::assert_cmd(&cmd, &fixture.root());

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
    common::assert_cmd(&cmd, &fixture.path("parent"));

    common::exec_garden(&["--chdir", &fixture.root(), "plant", "parent"])?;
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "child"])?;

    let garden_yaml = fixture.path("garden.yaml");
    let path = Some(std::path::PathBuf::from(&garden_yaml));

    let cfg = garden::config::new(&path, &fixture.root(), 0, None)?;
    assert_eq!(2, cfg.trees.len());
    assert_eq!("parent", cfg.trees[0].get_name());

    // The "child" repository is a child worktree of the "parent" tree.
    assert!(cfg.trees[1].is_worktree);
    assert_eq!(cfg.trees[1].worktree.get_expr(), "parent");
    assert_eq!(cfg.trees[1].branch.get_expr(), "dev");

    Ok(())
}
