pub mod common;

use anyhow::Result;

use garden::string;

#[test]
fn garden_root() -> Result<()> {
    // The test has garden.root = ${root}
    // with variables: src = src, and root = ~/${src}
    // This should expand to $HOME/src.
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    let expect_src_dir = string!("/home/test/src");
    assert_eq!("${root}", config.root.get_expr());
    assert_eq!(Some(&expect_src_dir), config.root.get_value());
    assert_eq!(expect_src_dir, config.root_path.to_string_lossy());

    Ok(())
}

#[test]
fn tree_variable() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    let tree_name = garden::model::TreeName::from("git");
    let result = garden::eval::tree_value(&config, "${prefix}", &tree_name, None);
    assert_eq!(result, "/home/test/.local");

    Ok(())
}

#[test]
fn config_variable() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    let tree_name = garden::model::TreeName::from("git");

    let test = garden::eval::tree_value(&config, "${test}", &tree_name, None);
    assert_eq!("TEST", test);

    let local = garden::eval::tree_value(&config, "${local}", &tree_name, None);
    assert_eq!("TEST/local", local);

    Ok(())
}

/// ${TREE_NAME} should be set to the current tree's name
#[test]
fn tree_name() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    let tree_name = garden::model::TreeName::from("git");
    let expect = "git";
    let actual = garden::eval::tree_value(&config, "${TREE_NAME}", &tree_name, None);
    assert_eq!(expect, actual);

    Ok(())
}

/// ${TREE_PATH} should be set to the current tree's path
#[test]
fn tree_path() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    let tree_name = garden::model::TreeName::from("git");
    let expect = "/home/test/src/git";
    let actual = garden::eval::tree_value(&config, "${TREE_PATH}", &tree_name, None);
    assert_eq!(expect, actual);

    Ok(())
}

/// ${GARDEN_ROOT} should be set to the garden.root configuration
#[test]
fn garden_path() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    let expect = "/home/test/src";
    let actual = garden::eval::value(&config, "${GARDEN_ROOT}");
    assert_eq!(expect, actual);

    Ok(())
}

#[test]
fn exec_expression() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();

    // Simple exec expression
    let value = garden::eval::value(&config, "$ echo test");
    assert_eq!(value, "test");

    // Exec expression found through variable indirection:
    //  ${echo_cmd} = "echo cmd"
    //  ${echo_cmd_exec} = "$ ${echo_cmd}"
    // Evaluation of ${echo_cmd_exec} produces "$ ${echo_cmd}"
    // which is further evaluated to "$ echo cmd" before getting
    // run through a shell to produce the final result.
    let value = garden::eval::value(&config, "${echo_cmd_exec}");
    assert_eq!(value, "cmd");

    // Ensure that exec expressions are evaluated in the tree directory.
    let context = garden::query::tree_context(&config, "tmp", None)?;
    let value = garden::eval::tree_value(&config, "$ echo $PWD", &context.tree, None);
    assert!(value == "/tmp" || value == "/private/tmp");

    let value = garden::eval::tree_value(&config, "$ pwd", &context.tree, None);
    assert!(value == "/tmp" || value == "/private/tmp");

    Ok(())
}

/// Ensure that shell $variables can be used.
#[test]
fn shell_variable_syntax() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();

    // Simple exec expression
    let value = garden::eval::value(&config, "$ value=$(echo test); echo $value");
    assert_eq!(value, "test");

    // Escaped ${braced} value
    let value = garden::eval::value(&config, "$ echo '$${value[@]:0:1}'");
    assert_eq!(value, "${value[@]:0:1}");

    Ok(())
}

#[test]
fn multi_variable_with_tree() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    assert!(config.trees.len() > 1);
    assert!(config.trees[1].environment.len() > 1);

    let mut var = config.trees[1].environment[1].clone();
    assert_eq!("PATH", var.get_name());

    let context = garden::model::TreeContext::new("cola", None, None, None);
    let values = garden::eval::multi_variable(&config, &mut var, &context);
    assert_eq!(
        values,
        [
            "/home/test/src/git-cola/local/bin",
            "/home/test/src/git-cola/bin",
        ]
    );

    Ok(())
}

#[test]
fn multi_variable_with_garden() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    assert!(config.trees.len() > 1);
    assert!(config.trees[1].environment.len() > 1);

    let mut var = config.trees[1].environment[1].clone();
    assert_eq!("PATH", var.get_name());

    let context = garden::model::TreeContext::new("cola", None, Some(string!("cola")), None);
    let values = garden::eval::multi_variable(&config, &mut var, &context);
    assert_eq!(
        values,
        [
            "/home/test/apps/git-cola/current/bin",
            "/home/test/src/git-cola/bin",
        ]
    );

    Ok(())
}

#[test]
fn garden_environment() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    // cola tree(1) and cola garden(Some(0))
    let context = garden::model::TreeContext::new("cola", None, Some(string!("cola")), None);
    let values = garden::eval::environment(&config, &context);
    assert_eq!(values.len(), 9);

    let mut idx = 0;
    assert_eq!(values[idx].0, "EXAMPLE_VALUE"); // ${TREE_PATH} for cola
    assert_eq!(values[idx].1, "/home/test/src");

    idx += 1;
    assert_eq!(values[idx].0, "PATH"); // ${TREE_PATH} for cola
    assert_eq!(values[idx].1, "/home/test/bin:/usr/bin:/bin");

    idx += 1;
    assert_eq!(values[idx].0, "PYTHONPATH"); // ${TREE_PATH} for cola
    assert_eq!(values[idx].1, "/home/test/src/git-cola");

    idx += 1;
    assert_eq!(values[idx].0, "PATH"); // cola ${TREE_PATH}/bin
    assert_eq!(
        values[idx].1,
        "/home/test/apps/git-cola/current/bin:/home/test/bin:/usr/bin:/bin"
    );

    idx += 1;
    assert_eq!(values[idx].0, "PATH"); // cola ${prefix}/bin garden prefix
    assert_eq!(
        values[idx].1,
        format!(
            "{}:{}:{}:/usr/bin:/bin",
            "/home/test/src/git-cola/bin", "/home/test/apps/git-cola/current/bin", "/home/test/bin"
        )
    );

    // cola tree ${GARDEN_ROOT}/python/send2trash, ${TREE_PATH}
    idx += 1;
    assert_eq!(values[idx].0, "PYTHONPATH");
    assert_eq!(
        values[idx].1,
        "/home/test/src/python/send2trash:/home/test/src/git-cola"
    );

    idx += 1;
    assert_eq!(values[idx].0, "PYTHONPATH"); // qtpy ${prefix}
    assert_eq!(
        values[idx].1,
        "/home/test/src/python/qtpy:/home/test/src/python/send2trash:/home/test/src/git-cola"
    );

    idx += 1;
    assert_eq!(values[idx].0, "GIT_COLA_TRACE"); // cola garden GIT_COLA_TRACE=: full
    assert_eq!(values[idx].1, "full");

    idx += 1;
    assert_eq!(values[idx].0, "PATH"); // cola garden ${prefix}/bin
    assert_eq!(
        values[idx].1,
        format!(
            "{}:{}:{}:/usr/bin:/bin:{}",
            "/home/test/src/git-cola/bin",
            "/home/test/apps/git-cola/current/bin",
            "/home/test/bin",
            "/home/test/apps/git-cola/current/bin"
        )
    );

    idx += 1;
    assert_eq!(values.len(), idx);

    Ok(())
}

#[test]
fn group_environment() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    // cola tree(1) + cola group(Some(0))
    let context = garden::model::TreeContext::new("cola", None, None, Some(string!("cola")));
    let values = garden::eval::environment(&config, &context);
    assert_eq!(values.len(), 7);

    let mut idx = 0;
    assert_eq!(values[idx].0, "EXAMPLE_VALUE");
    assert_eq!(values[idx].1, "/home/test/src");

    idx += 1;
    assert_eq!(values[idx].0, "PATH");
    assert_eq!(values[idx].1, "/home/test/bin:/usr/bin:/bin");

    // ${TREE_PATH} for cola
    idx += 1;
    assert_eq!(values[idx].0, "PYTHONPATH");
    assert_eq!(values[idx].1, "/home/test/src/git-cola");

    // cola tree ${prefix}/bin
    idx += 1;
    assert_eq!(values[idx].0, "PATH");
    assert_eq!(
        values[idx].1,
        "/home/test/src/git-cola/local/bin:/home/test/bin:/usr/bin:/bin"
    );

    // cola tree ${TREE_PATH}/bin
    idx += 1;
    assert_eq!(values[idx].0, "PATH");
    assert_eq!(
        values[idx].1,
        "/home/test/src/git-cola/bin:/home/test/src/git-cola/local/bin:/home/test/bin:/usr/bin:/bin"
    );

    // cola tree ${GARDEN_ROOT}/python/send2trash
    idx += 1;
    assert_eq!(values[idx].0, "PYTHONPATH");
    assert_eq!(
        values[idx].1,
        "/home/test/src/python/send2trash:/home/test/src/git-cola"
    );

    // qtpy ${prefix}
    idx += 1;
    assert_eq!(values[idx].0, "PYTHONPATH");
    assert_eq!(
        values[idx].1,
        format!(
            "{}:{}:{}",
            "/home/test/src/python/qtpy",
            "/home/test/src/python/send2trash",
            "/home/test/src/git-cola"
        )
    );

    idx += 1;
    assert_eq!(values.len(), idx);

    Ok(())
}

#[test]
fn environment_empty_value() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    let context = garden::query::tree_from_name(&config, "tmp", None, None).unwrap();
    let values = garden::eval::environment(&config, &context);
    assert_eq!(values.len(), 5);

    let mut idx = 0;
    assert_eq!(values[idx].0, "EXAMPLE_VALUE");
    assert_eq!(values[idx].1, "/home/test/src");

    idx += 1;
    assert_eq!(values[idx].0, "PATH");
    assert_eq!(values[idx].1, "/home/test/bin:/usr/bin:/bin");

    idx += 1;
    assert_eq!(values[idx].0, "EMPTY"); // prepend "a", must not have a ":"
    assert_eq!(values[idx].1, "a");

    idx += 1;
    assert_eq!(values[idx].0, "EMPTY"); // prepend "b", must have ":"
    assert_eq!(values[idx].1, "b:a");

    idx += 1;
    assert_eq!(values[idx].0, "tmp_VALUE"); // ${TREE_NAME}_VALUE: ${TREE_PATH}
    assert_eq!(values[idx].1, "/tmp");

    idx += 1;
    assert_eq!(values.len(), idx);

    Ok(())
}

#[test]
fn command_garden_scope() -> Result<()> {
    let app_context = common::garden_context()?;
    let context = garden::model::TreeContext::new("cola", None, Some(string!("cola")), None);

    // Garden scope
    let values = garden::eval::command(&app_context, &context, "build");
    assert_eq!(values.len(), 1);

    let cmd_vec = &values[0];
    assert_eq!(cmd_vec.len(), 1);

    let cmd = &cmd_vec[0];
    assert_eq!(cmd, "make -j prefix=/home/test/apps/git-cola/current all");

    Ok(())
}

#[test]
fn command_tree_scope() -> Result<()> {
    let app_context = common::garden_context()?;
    let context = garden::model::TreeContext::new("cola", None, None, None);

    // The ${prefix} variable should expand to the tree-local value.
    {
        let values = garden::eval::command(&app_context, &context, "build");
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].len(), 1);

        let cmd = &values[0][0];
        assert_eq!(cmd, "make -j prefix=/home/test/src/git-cola/local all");
    }

    // Commands should include the template commands followed by the
    // tree-specific commands.
    {
        let values = garden::eval::command(&app_context, &context, "test");
        assert_eq!(values.len(), 1);

        assert_eq!(values[0].len(), 2);
        assert_eq!(values[0][0], "git status --short");
        assert_eq!(values[0][1], "make tox");
    }

    Ok(())
}

#[test]
fn environment_variables() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    // Environment variables in tree scope
    std::env::set_var("GARDEN_TEST_VALUE", "test");

    let value = garden::eval::value(&config, "${GARDEN_TEST_VALUE}");
    assert_eq!(value, "test");

    let value = garden::eval::tree_value(&config, "${GARDEN_TEST_VALUE}", "git", None);
    assert_eq!(value, "test");

    Ok(())
}

#[test]
fn find_tree_in_graft() -> Result<()> {
    // See the "config.rs" tests for config-level validations.
    let app = garden::model::ApplicationContext::from_path_string("tests/data/garden.yaml")?;
    let id = app.get_root_id();
    let ctx = garden::query::find_tree(&app, id, "graft::graft", None)?;
    assert_eq!("graft", ctx.tree);
    assert!(ctx.config.is_some());

    let node_id: usize = ctx.config.unwrap().into();
    assert_eq!(2usize, node_id);

    Ok(())
}

#[test]
fn eval_graft_tree() -> Result<()> {
    let app = garden::model::ApplicationContext::from_path_string("tests/data/garden.yaml")?;
    let id = app.get_root_id();

    // Get a tree context for "graft::graft" from the outer-most config.
    let ctx = garden::query::find_tree(&app, id, "graft::graft", None)?;
    assert!(ctx.config.is_some());

    let node_id: usize = ctx.config.unwrap().into();
    assert_eq!(2usize, node_id);

    // Evaluate the value for ${current_config} using the inner grafted config.
    let config = app.get_config(ctx.config.unwrap());
    let path = garden::eval::tree_value(config, "${TREE_PATH}", &ctx.tree, ctx.garden.as_ref());
    assert!(path.ends_with("/graft"));

    // Evaluate a local variable that is overridden in the graft.
    let actual =
        garden::eval::tree_value(config, "${current_config}", &ctx.tree, ctx.garden.as_ref());
    assert_eq!("graft", actual);

    // Get a TreeContext for "example/tree".
    let example_ctx = garden::query::find_tree(&app, id, "example/tree", None)?;
    assert!(example_ctx.config.is_some());

    // Get the configuration for "example/tree".
    let example_config = app.get_config(example_ctx.config.unwrap());
    // Evaluate "${current_config}" from the context of "example/tree".
    let actual = garden::eval::tree_value(
        example_config,
        "${current_config}",
        &example_ctx.tree,
        example_ctx.garden.as_ref(),
    );
    assert_eq!("main", actual);

    // References to unknown grafts evaluate to an empty string.
    let actual = garden::eval::tree_value(
        config,
        "${undefined::variable}",
        &ctx.tree,
        ctx.garden.as_ref(),
    );
    assert_eq!("", actual);

    // Evaluate a grafted variable from the context of "example/tree" from
    // the main configuration.
    let actual = garden::eval::tree_value(
        config,
        "${graft::current_config}",
        &ctx.tree,
        ctx.garden.as_ref(),
    );
    // TODO: this should evaluate to "graft".
    //assert_eq!("graft", actual);
    assert_eq!("", actual);

    Ok(())
}

#[test]
fn eval_graft_variables() -> Result<()> {
    let app_context = garden::model::ApplicationContext::from_path_string("tests/data/garden.yaml")?;
    let config = app_context.get_root_config();

    // Evaluate graft variables one level deep.
    let actual = garden::eval::get_value(&app_context, config, "${graft::current_config}");
    assert_eq!("graft", actual);

    let actual = garden::eval::get_value(&app_context, config, "${graft::variable}");
    assert_eq!("graft value", actual);

    // Evaluate graft variables two levels deep.
    let actual = garden::eval::get_value(&app_context, config, "${graft::deps::current_config}");
    assert_eq!("deps", actual);

    let actual = garden::eval::get_value(&app_context, config, "${graft::deps::deps_graft_value}");
    assert_eq!("deps-graft-value", actual);

    Ok(())
}
