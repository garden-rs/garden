extern crate garden;
extern crate dirs;

mod common;


#[test]
fn garden_root () {
    // The test has garden.root = ${root}
    // with variables: src = src, and root = ~/${src}
    // This should expand to $HOME/src.
    let mut path_buf = dirs::home_dir().unwrap();
    path_buf.push("src");
    let expect_src_dir = path_buf.to_string_lossy();

    let mut config = common::garden_config();
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

    let path_buf = dirs::home_dir().unwrap();
    let home_dir= path_buf.to_string_lossy();

    assert!(result.starts_with(home_dir.as_ref()));
    assert!(result.ends_with("/.local"));
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