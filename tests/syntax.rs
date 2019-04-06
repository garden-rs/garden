extern crate garden;

use garden::syntax;


#[test]
fn is_garden() {
    assert!(syntax::is_garden(":garden"), ":garden is a garden");
    assert!(!syntax::is_garden("garden"), "garden is not a garden");
}


#[test]
fn is_graft() {
    assert!(syntax::is_graft("foo::bar"), "foo::bar is a graft");
    assert!(!syntax::is_graft("foo"), "foo is not a graft");
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


#[test]
fn split_string_ok() {
    let (ok, pre, post) = syntax::split_string("foo::bar", "::");
    assert!(ok, "split :: on foo::bar is ok");
    assert_eq!(pre, "foo");
    assert_eq!(post, "bar");
}


#[test]
fn split_string_empty() {
    let (ok, pre, post) = syntax::split_string("foo::", "::");
    assert!(ok, "split :: on foo:: is ok");
    assert_eq!(pre, "foo");
    assert_eq!(post, "");
}


#[test]
fn split_string_not_found() {
    let (ok, pre, post) = syntax::split_string("foo", "::");
    assert!(!ok, "split :: on foo is false");
    assert_eq!(pre, "foo");
    assert_eq!(post, "");
}
