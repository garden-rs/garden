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
fn is_git_dir() {
    assert!(syntax::is_git_dir("tree.git"), "tree.git is a git dir");
    assert!(syntax::is_git_dir("/src/tree.git"), "/src/tree.git is a git dir");
    assert!(!syntax::is_git_dir("src/tree/.git"), "src/tree/.git is a git dir");
    assert!(!syntax::is_git_dir(".git"), ".git is a git dir");
    assert!(!syntax::is_git_dir("/.git"), "/.git is a git dir");
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

#[test]
fn split_graft_ok() {
    let (ok, pre, post) = syntax::split_graft("foo::bar");
    assert!(ok, "split_graft on foo::bar is ok");
    assert_eq!(pre, "foo");
    assert_eq!(post, "bar");
}

#[test]
fn split_graft_nested_ok() {
    let (ok, pre, post) = syntax::split_graft("@foo::bar::baz");
    assert!(ok, "split_graft on @foo::bar::baz is ok");
    assert_eq!(pre, "@foo");
    assert_eq!(post, "bar::baz");
}

#[test]
fn split_graft_empty() {
    let (ok, pre, post) = syntax::split_graft("foo::");
    assert!(ok, "split_graft on foo:: is ok");
    assert_eq!(pre, "foo");
    assert_eq!(post, "");
}

#[test]
fn split_graft_not_found() {
    let (ok, pre, post) = syntax::split_graft("foo");
    assert!(!ok, "split_graft on foo is false");
    assert_eq!(pre, "foo");
    assert_eq!(post, "");
}

#[test]
fn trim_exec() {
    assert_eq!("cmd", syntax::trim_exec("$ cmd"));
    assert_eq!("$cmd", syntax::trim_exec("$cmd"));
    assert_eq!("cmd", syntax::trim_exec("cmd"));
    assert_eq!("", syntax::trim_exec("$ "));
    assert_eq!("$", syntax::trim_exec("$"));
    assert_eq!("", syntax::trim_exec(""));
}

#[test]
fn trim_graft() {
    let value = syntax::trim_graft("foo::bar::baz");
    assert!(value.is_some());
    assert_eq!("bar::baz", value.unwrap());

    let value = syntax::trim_graft("@foo::bar::baz");
    assert!(value.is_some());
    assert_eq!("@bar::baz", value.unwrap());

    let value = syntax::trim_graft("%foo::bar::baz");
    assert!(value.is_some());
    assert_eq!("%bar::baz", value.unwrap());

    let value = syntax::trim_graft(":foo::bar::baz");
    assert!(value.is_some());
    assert_eq!(":bar::baz", value.unwrap());

    let value = syntax::trim_graft("foo::bar");
    assert!(value.is_some());
    assert_eq!("bar", value.unwrap());

    let value = syntax::trim_graft("foo");
    assert!(value.is_none());
}

#[test]
fn graft_basename() {
    let value = syntax::graft_basename("foo");
    assert!(value.is_none());

    let value = syntax::graft_basename(":foo");
    assert!(value.is_none());

    let value = syntax::graft_basename("%foo");
    assert!(value.is_none());

    let value = syntax::graft_basename("@foo");
    assert!(value.is_none());

    let value = syntax::graft_basename("foo::bar");
    assert!(value.is_some());
    assert_eq!("foo", value.unwrap());

    let value = syntax::graft_basename(":foo::bar");
    assert!(value.is_some());
    assert_eq!("foo", value.unwrap());

    let value = syntax::graft_basename("%foo::bar");
    assert!(value.is_some());
    assert_eq!("foo", value.unwrap());

    let value = syntax::graft_basename("@foo::bar");
    assert!(value.is_some());
    assert_eq!("foo", value.unwrap());

    let value = syntax::graft_basename("foo::bar::baz");
    assert!(value.is_some());
    assert_eq!("foo", value.unwrap());

    let value = syntax::graft_basename(":foo::bar::baz");
    assert!(value.is_some());
    assert_eq!("foo", value.unwrap());

    let value = syntax::graft_basename("%foo::bar::baz");
    assert!(value.is_some());
    assert_eq!("foo", value.unwrap());

    let value = syntax::graft_basename("@foo::bar::baz");
    assert!(value.is_some());
    assert_eq!("foo", value.unwrap());
}
