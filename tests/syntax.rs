extern crate garden;

use garden::syntax;


#[test]
fn is_garden() {
    assert!(syntax::is_garden(":garden"), ":garden is a garden");
    assert!(!syntax::is_garden("garden"), "garden is not a garden");
}


#[test]
fn is_group() {
    assert!(syntax::is_group("%group"), "%group is a group");
    assert!(!syntax::is_group("group"), "group is not a group");
}


#[test]
fn is_tree() {
    assert!(syntax::is_tree("@tree"), "@tree is a tree");
    assert!(!syntax::is_tree("tree"), "tree is not a tree");
}
