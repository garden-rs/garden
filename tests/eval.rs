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


/// ${TREE_NAME} should be set to the current tree's name
#[test]
fn tree_name() {
    let mut config = common::garden_config();
    let tree_idx: garden::model::TreeIndex = 0;
    let expect = "git";
    let actual = garden::eval::tree_value(
        &mut config, "${TREE_NAME}", tree_idx, None);
    assert_eq!(expect, actual);
}


/// ${TREE_PATH} should be set to the current tree's path
#[test]
fn tree_path() {
    let mut config = common::garden_config();
    let tree_idx: garden::model::TreeIndex = 0;
    let expect = "/home/test/src/git";
    let actual = garden::eval::tree_value(
        &mut config, "${TREE_PATH}", tree_idx, None);
    assert_eq!(expect, actual);
}


/// ${GARDEN_ROOT} should be set to the garden.root configuration
#[test]
fn garden_path() {
    let mut config = common::garden_config();
    let expect = "/home/test/src";
    let actual = garden::eval::value(&mut config, "${GARDEN_ROOT}");
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
        group: None,
    };
    let values = garden::eval::multi_variable(
        &mut config, &mut var, &context);
    assert_eq!(values,
               ["/home/test/src/git-cola/bin",
                "/home/test/src/git-cola/local/bin"]);
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
        group: None,
    };
    let values = garden::eval::multi_variable(
        &mut config, &mut var, &context);
    assert_eq!(values,
               ["/home/test/src/git-cola/bin",
                "/home/test/apps/git-cola/current/bin"]);
}


#[test]
fn environment() {
    let mut config = common::garden_config();
    let context = garden::model::TreeContext {
        tree: 1,  // cola
        garden: Some(0),
        group: None,
    };
    let values = garden::eval::environment(&mut config, &context);
    assert_eq!(values.len(), 7);

    let mut idx = 0;
    assert_eq!(values[idx].0, "PYTHONPATH");  // ${TREE_PATH} for cola
    assert_eq!(values[idx].1, "/home/test/src/git-cola");

    idx += 1;
    assert_eq!(values[idx].0, "PATH");  // cola tree ${TREE_PATH}/bin
    assert_eq!(values[idx].1, "/home/test/src/git-cola/bin:/usr/bin:/bin");

    idx += 1;
    assert_eq!(values[idx].0, "PATH");  // cola garden ${prefix}
    assert_eq!(values[idx].1,
               format!("{}:{}:/usr/bin:/bin",
                       "/home/test/apps/git-cola/current/bin",
                       "/home/test/src/git-cola/bin"));

    // cola tree ${GARDEN_ROOT}/python/send2trash, ${TREE_PATH}
    idx += 1;
    assert_eq!(values[idx].0, "PYTHONPATH");
    assert_eq!(values[idx].1,
               "/home/test/src/python/send2trash:/home/test/src/git-cola");

    idx += 1;
    assert_eq!(values[idx].0, "PYTHONPATH");  // qtpy ${prefix}
    assert_eq!(values[idx].1,
               "/home/test/src/python/qtpy:/home/test/src/python/send2trash:/home/test/src/git-cola");

    idx += 1;
    assert_eq!(values[idx].0, "GIT_COLA_TRACE");  // cola garden GIT_COLA_TRACE=: full
    assert_eq!(values[idx].1, "full");

    idx += 1;
    assert_eq!(values[idx].0, "PATH");  // coal garden ${prefix}/bin
    assert_eq!(values[idx].1,
               format!("{}:{}:/usr/bin:/bin:{}",
                       "/home/test/apps/git-cola/current/bin",
                       "/home/test/src/git-cola/bin",
                       "/home/test/apps/git-cola/current/bin"));

    idx += 1;
    assert_eq!(values.len(), idx);
}

#[test]
fn environment_empty_value() {
    let mut config = common::garden_config();
    let context = garden::query::tree_from_name(&config, "tmp", None).unwrap();
    let values = garden::eval::environment(&mut config, &context);
    assert_eq!(values.len(), 3);

    let mut idx = 0;
    assert_eq!(values[idx].0, "EMPTY");  // prepend "a", must not have a ":"
    assert_eq!(values[idx].1, "a");

    idx += 1;
    assert_eq!(values[idx].0, "EMPTY");  // prepend "b", must have ":"
    assert_eq!(values[idx].1, "b:a");

    idx += 1;
    assert_eq!(values[idx].0, "tmp_VALUE");  // ${TREE_NAME}_VALUE: ${TREE_PATH}
    assert_eq!(values[idx].1, "/tmp");

    idx += 1;
    assert_eq!(values.len(), idx);
}


#[test]
fn command_garden_scope() {
    let mut config = common::garden_config();
    let context = garden::model::TreeContext {
        tree: 1,
        garden: Some(0),
        group: None,
    };

    // Garden scope
    let values = garden::eval::command(&mut config, &context, "build");
    assert_eq!(values.len(), 1);

    let cmd_vec = &values[0];
    assert_eq!(cmd_vec.len(), 1);

    let cmd = &cmd_vec[0];
    assert_eq!(cmd, "make -j prefix=/home/test/apps/git-cola/current all");
}


#[test]
fn command_tree_scope() {
    let mut config = common::garden_config();
    let context = garden::model::TreeContext {
        tree: 1,
        garden: None,
        group: None,
    };

    // The ${prefix} variable should expand to the tree-local value.
    {
        let values = garden::eval::command(&mut config, &context, "build");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].len(), 1);

        let cmd = &values[0][0];
        assert_eq!(cmd, "make -j prefix=/home/test/src/git-cola/local all");
    }

    // Commands should include the template commands followed by the
    // tree-specific commands.
    {
        let values = garden::eval::command(&mut config, &context, "test");
        assert_eq!(values.len(), 2);

        assert_eq!(values[0].len(), 1);
        assert_eq!(values[0][0], "make test");

        assert_eq!(values[1].len(), 2);
        assert_eq!(values[1][0], "git status --short");
        assert_eq!(values[1][1], "make tox");

    }
}


#[test]
fn environment_variables() {
    let mut config = common::garden_config();
    // Environment variables in tree scope
    std::env::set_var("GARDEN_TEST_VALUE", "test");

    let mut value = garden::eval::value(&mut config, "${GARDEN_TEST_VALUE}");
    assert_eq!(value, "test");

    value = garden::eval::tree_value(
        &mut config, "${GARDEN_TEST_VALUE}", 0, None);
    assert_eq!(value, "test");
}
