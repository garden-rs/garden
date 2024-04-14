/// Integration tests for the garden::query module.
///
/// These tests use the BareRepoFixture module and cannot be used used alongside tests
/// that call common::garden_context(), which calls common::initialize_environment(),
/// because those functions modify $PATH, which breaks Command::cargo_bin("garden").
pub mod common;

use anyhow::Result;
use function_name::named;

use garden::string;

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "powerpc64",
    target_arch = "s390x",
    target_arch = "x86"
)))]
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
    let pathbuf = std::path::PathBuf::from("tests/data/worktree.yaml");
    let app_context = garden::model::ApplicationContext::from_path_and_root(
        &pathbuf,
        Some(&fixture.root_pathbuf()),
    )?;
    let cfg = app_context.get_root_config();

    let tree_name = garden::query::tree_name_from_path(cfg, &fixture.worktree_pathbuf("dev"));
    assert_eq!(tree_name, Some(string!("dev")));

    let tree_name = garden::query::tree_name_from_path(cfg, &fixture.worktree_pathbuf("default"));
    assert_eq!(tree_name, Some(string!("default")));

    Ok(())
}
