extern crate garden;

mod common;

#[test]
fn default_expression_finds_garden() {
    let config = common::garden_config();
    let result = garden::query::resolve_trees(&config, "cola");
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].garden, Some(0));
    assert_eq!(result[0].tree, 0);
    assert_eq!(result[1].garden, Some(0));
    assert_eq!(result[1].tree, 1);
    assert_eq!(result[2].garden, Some(0));
    assert_eq!(result[2].tree, 2);
}


#[test]
fn tree_expression_wildcard() {
    let config = common::garden_config();
    let result = garden::query::resolve_trees(&config, "@c*");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].garden, None);
    assert_eq!(result[0].group, None);
    assert_eq!(result[0].tree, 1);
}


#[test]
fn group_expression() {
    let config = common::garden_config();
    let result = garden::query::resolve_trees(&config, "%rev*");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].garden, None);
    assert_eq!(result[0].group, Some(2));
    assert_eq!(result[0].tree, 1);
    assert_eq!(result[1].garden, None);
    assert_eq!(result[1].group, Some(2));
    assert_eq!(result[1].tree, 0);
}

#[test]
fn trees_from_pattern() {
    let config = common::garden_config();
    let result = garden::query::trees_from_pattern(&config, "annex/*", None, None);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].garden, None);
    assert_eq!(result[0].group, None);
    assert_eq!(result[0].tree, 4);  // annex/data
    assert_eq!(result[1].garden, None);
    assert_eq!(result[1].group, None);
    assert_eq!(result[1].tree, 5);  // annex/local
}

#[test]
fn trees_from_group() {
    let config = common::garden_config();
    assert!(config.groups.len() > 3);

    let annex_grp = &config.groups[3];
    assert_eq!(annex_grp.name, "annex");

    let result = garden::query::trees_from_group(&config, annex_grp);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].garden, None);
    assert_eq!(result[0].group, Some(3));
    assert_eq!(result[0].tree, 4);  // annex/data
    assert_eq!(result[1].garden, None);
    assert_eq!(result[1].group, Some(3));
    assert_eq!(result[1].tree, 5);  // annex/local
}
