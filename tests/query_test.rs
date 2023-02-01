/// Tests for the garden::query module.
///
/// These tests modify global environment variables (eg. $PATH) and cannot
/// be used alongside tests that use BareRepoFixture.
pub mod common;

use anyhow::{Context, Result};

use garden::string;

#[test]
fn resolve_trees_default_query_finds_garden() {
    let config = common::garden_config();
    let result = garden::query::resolve_trees(&config, "cola");
    assert_eq!(3, result.len());
    assert_eq!(Some(string!("cola")), result[0].garden);
    assert_eq!(0, result[0].tree);
    assert_eq!(Some(string!("cola")), result[1].garden);
    assert_eq!(1, result[1].tree);
    assert_eq!(Some(string!("cola")), result[2].garden);
    assert_eq!(2, result[2].tree);
}

#[test]
fn resolve_trees_tree_query_wildcard() {
    let config = common::garden_config();
    let result = garden::query::resolve_trees(&config, "@c*");
    assert_eq!(1, result.len());
    assert_eq!(None, result[0].garden);
    assert_eq!(None, result[0].group);
    assert_eq!(1, result[0].tree);
}

#[test]
fn resolve_trees_group_query() {
    let config = common::garden_config();
    let result = garden::query::resolve_trees(&config, "%rev*");
    assert_eq!(2, result.len());
    assert_eq!(None, result[0].garden);
    assert_eq!(Some(string!("reverse")), result[0].group);
    assert_eq!(1, result[0].tree);
    assert_eq!(None, result[1].garden);
    assert_eq!(Some(string!("reverse")), result[1].group);
    assert_eq!(0, result[1].tree);
}

#[test]
fn resolve_trees_group_with_wildcards() {
    let config = common::garden_config();
    // annex group
    let result = garden::query::resolve_trees(&config, "%annex");
    assert_eq!(2, result.len());
    // annex/data
    assert_eq!(None, result[0].garden);
    assert_eq!(Some(string!("annex")), result[0].group);
    assert_eq!(4, result[0].tree);
    // annex/local
    assert_eq!(None, result[1].garden);
    assert_eq!(Some(string!("annex")), result[1].group);
    assert_eq!(5, result[1].tree);
}

#[test]
fn trees_from_pattern() {
    let config = common::garden_config();
    let result = garden::query::trees_from_pattern(&config, "annex/*", None, None);
    assert_eq!(2, result.len());
    assert_eq!(None, result[0].garden);
    assert_eq!(None, result[0].group);
    assert_eq!(4, result[0].tree); // annex/data
    assert_eq!(None, result[1].garden);
    assert_eq!(None, result[1].group);
    assert_eq!(5, result[1].tree); // annex/local
}

#[test]
fn trees_from_group() -> Result<()> {
    let config = common::garden_config();
    assert!(config.groups.len() > 3);

    let annex_grp = config.groups.get("annex").context("Missing annex group")?;
    assert_eq!("annex", annex_grp.get_name());

    let result = garden::query::trees_from_group(&config, None, annex_grp);
    assert_eq!(2, result.len());
    assert_eq!(None, result[0].garden);
    assert_eq!(Some(string!("annex")), result[0].group);
    assert_eq!(4, result[0].tree); // annex/data
    assert_eq!(None, result[1].garden);
    assert_eq!(Some(string!("annex")), result[1].group);
    assert_eq!(5, result[1].tree); // annex/local

    Ok(())
}

#[test]
fn trees_from_garden() -> Result<()> {
    let config = common::garden_config();
    assert!(config.gardens.len() > 3);

    // regular group, group uses wildcards
    let annex_group = config.gardens.get("annex/group").context("annex/group")?;
    let mut result = garden::query::trees_from_garden(&config, annex_group);
    assert_eq!(2, result.len());

    // annex/group
    assert_eq!(Some(string!("annex/group")), result[0].garden);
    assert_eq!(Some(string!("annex")), result[0].group);
    assert_eq!(4, result[0].tree);
    // annex/group
    assert_eq!(Some(string!("annex/group")), result[1].garden);
    assert_eq!(Some(string!("annex")), result[1].group);
    assert_eq!(5, result[1].tree);

    // wildcard groups
    let annex_wild_groups = config
        .gardens
        .get("annex/wildcard-groups")
        .context("annex/wildcard-groups")?;
    result = garden::query::trees_from_garden(&config, annex_wild_groups);
    assert_eq!(2, result.len());

    // annex/Wildcard-groups
    assert_eq!(Some(string!("annex/wildcard-groups")), result[0].garden);
    assert_eq!(Some(string!("annex-1")), result[0].group);
    assert_eq!(4, result[0].tree);

    assert_eq!(Some(string!("annex/wildcard-groups")), result[1].garden);
    assert_eq!(Some(string!("annex-2")), result[1].group);
    assert_eq!(5, result[1].tree);

    // wildcard trees
    let annex_wild_trees = config
        .gardens
        .get("annex/wildcard-trees")
        .context("annex/wildcard-trees")?;
    result = garden::query::trees_from_garden(&config, annex_wild_trees);
    assert_eq!(2, result.len());

    assert_eq!(Some(string!("annex/wildcard-trees")), result[0].garden);
    assert_eq!(None, result[0].group);
    assert_eq!(4, result[0].tree);

    assert_eq!(Some(string!("annex/wildcard-trees")), result[1].garden);
    assert_eq!(None, result[1].group);
    assert_eq!(5, result[1].tree);

    Ok(())
}

#[test]
fn tree_query() {
    let config = common::garden_config();

    // Success: "cola" is in the "git" garden.
    let tree_context_result = garden::query::tree_context(&config, "cola", Some("git"));
    assert!(tree_context_result.is_ok());
    let tree_context = tree_context_result.unwrap();
    assert_eq!(1, tree_context.tree);
    assert_eq!(Some(string!("git")), tree_context.garden);

    // Success: "cola" alone has a tree context with a None garden.
    let tree_context_result = garden::query::tree_context(&config, "cola", None);
    assert!(tree_context_result.is_ok());
    let tree_context = tree_context_result.unwrap();
    assert_eq!(1, tree_context.tree);
    assert_eq!(None, tree_context.garden);

    // "unknown" tarden is not a real garden.
    let tree_context_result = garden::query::tree_context(&config, "cola", Some("unknown-garden"));
    assert!(tree_context_result.is_err());

    // "tmp" is not in the "git" garden, and must raise an error.
    let tree_context_result = garden::query::tree_context(&config, "tmp", Some("git"));
    assert!(tree_context_result.is_err());

    // "unknown-tree" is not a real tree.
    let tree_context_result = garden::query::tree_context(&config, "unknown-tree", None);
    assert!(tree_context_result.is_err());
}
