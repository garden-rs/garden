/// Integration tests for the garden::query module.
///
/// These tests use the BareRepoFixture module and cannot be used used alongside tests
/// that call common::garden_config(), which calls common::initialize_environment(),
/// because those functions modify $PATH, which breaks Command::cargo_bin("garden").
pub mod common;

use anyhow::Result;
use function_name::named;

#[test]
#[named]
fn tree_name_from_pathbuf() -> Result<()> {
    let fixture = common::BareRepoFixture::new(function_name!());
    // garden grow dev
    common::exec_garden(&[
        "--chdir",
        &fixture.root(),
        "--config",
        "tests/data/worktree.yaml",
        "grow",
        "dev",
    ])?;
    // Growing "dev" should have also grown the "default" parent worktree.
    // Load the configuration and lookup the trees from the path.
    let path = Some(std::path::PathBuf::from("tests/data/worktree.yaml"));
    let cfg = garden::config::new(&path, &Some(fixture.root_pathbuf()), 0, None)?;

    let tree_name = garden::query::tree_name_from_path(&cfg, &fixture.worktree_pathbuf("dev"));
    assert_eq!(tree_name, Some("dev".to_string()));

    let tree_name = garden::query::tree_name_from_path(&cfg, &fixture.worktree_pathbuf("default"));
    assert_eq!(tree_name, Some("default".to_string()));

    Ok(())
}
