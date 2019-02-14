extern crate garden;
extern crate dirs;

mod common;


#[test]
fn garden_root () {
    // The test has garden.root = ${root}
    // with variables: src = src, and root = ~/${src}
    // This should expand to $HOME/src.
    let config = common::garden_config();
    let expect_src_dir = "/home/test/src";
    assert_eq!(config.root.expr, "${root}");
    assert_eq!(config.root.value.unwrap(), expect_src_dir);
    assert_eq!(config.root_path.to_string_lossy(), expect_src_dir);
}

#[test]
fn tree_variable() {
    let mut config = common::garden_config();
    let tree_idx: garden::model::TreeIndex = 0;
    let result = garden::eval::tree_value(
        &mut config, "${prefix}", tree_idx, None);
    assert_eq!(result, "/home/test/.local");
}

#[test]
fn config_variable() {
    let mut config = common::garden_config();
    let tree_idx: garden::model::TreeIndex = 0;

    let test = garden::eval::tree_value(
        &mut config, "${test}", tree_idx, None);
    assert_eq!("TEST", test);

    let local = garden::eval::tree_value(
        &mut config, "${local}", tree_idx, None);
    assert_eq!("TEST/local", local);
}

#[test]
fn tree_path_variable() {
    let mut config = common::garden_config();
    let tree_idx: garden::model::TreeIndex = 0;
    let expect = "/home/test/src/git";
    let actual = garden::eval::tree_value(
        &mut config, "${TREE_PATH}", tree_idx, None);
    assert_eq!(expect, actual);
}


#[test]
fn exec_expression() {
    let mut config = common::garden_config();

    // Simple exec expression
    let mut value = garden::eval::value(&mut config, "$ echo test");
    assert_eq!(value, "test");

    // Exec expression found through variable indirection:
    //  ${echo_cmd} = "echo cmd"
    //  ${echo_cmd_exec} = "$ ${echo_cmd}"
    // Evaluation of ${echo_cmd_exec} produces "$ ${echo_cmd}"
    // which is further evaluated to "$ echo cmd" before getting
    // run through a shell to produce the final result.
    value = garden::eval::value(&mut config, "${echo_cmd_exec}");
    assert_eq!(value, "cmd");
}


#[test]
fn multi_variable_with_tree() {
    let mut config = common::garden_config();
    assert!(config.trees.len() > 1);
    assert!(config.trees[1].environment.len() > 1);

    let mut var = config.trees[1].environment[1].clone();
    assert_eq!(var.name, "PATH");

    let context = garden::model::TreeContext {
        tree: 1,
        garden: None,
    };
    let values = garden::eval::multi_variable(
        &mut config, &mut var, &context);
    assert_eq!(values,
               ["/home/test/src/git-cola/bin",
                "/home/test/src/git-cola/local"]);
}

#[test]
fn multi_variable_with_garden() {
    let mut config = common::garden_config();
    assert!(config.trees.len() > 1);
    assert!(config.trees[1].environment.len() > 1);

    let mut var = config.trees[1].environment[1].clone();
    assert_eq!(var.name, "PATH");

    let context = garden::model::TreeContext {
        tree: 1,
        garden: Some(0),
    };
    let values = garden::eval::multi_variable(
        &mut config, &mut var, &context);
    assert_eq!(values,
               ["/home/test/src/git-cola/bin",
                "/home/test/apps/git-cola/current"]);
}


#[test]
fn environment() {
    let mut config = common::garden_config();
    let context = garden::model::TreeContext {
        tree: 1,
        garden: Some(0),
    };
    let values = garden::eval::environment(&mut config, &context);
    assert_eq!(values.len(), 6);

    assert_eq!(values[0].0, "PYTHONPATH");
    assert_eq!(values[0].1, "/home/test/src/git-cola");

    assert_eq!(values[5].0, "PATH");

    assert!(values[5].1.starts_with(
        "/home/test/apps/git-cola/current:/home/test/src/git-cola/bin:"));

    assert!(values[5].1.ends_with(":/home/test/apps/git-cola/current"));
}
