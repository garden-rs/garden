use anyhow::Result;

#[test]
fn read_includes() -> Result<()> {
    let app = garden::build::context_from_path("tests/data/garden.yaml")?;
    let config = app.get_root_config();

    // var_0 is from the included variables.yaml..
    let actual = garden::eval::value(config, "${var_0}");
    assert_eq!(actual, "zero");
    // var_1 is provided by variables-transitive.yaml and overridden by includes.yaml.
    let actual = garden::eval::value(config, "${var_1}");
    assert_eq!(actual, "ONE");
    // var_2 is provided by variables-transitive.yaml.
    let actual = garden::eval::value(config, "${var_2}");
    assert_eq!(actual, "two");

    // Trees are provided by included configs.
    assert!(config.trees.len() >= 2);
    // trees[0] is included from trees.yaml.
    assert_eq!(config.trees[0].get_name(), "tree-zero");
    // trees[1] is from the main config.
    assert_eq!(config.trees[1].get_name(), "example/tree");

    // Nested include files are relative to the file that included them.
    // If the nested include file is not found relative to the parent include file
    // then a file relative to the config directory can be used.
    let actual = garden::eval::value(config, "${var_included}");
    assert_eq!(actual, "relative to config");

    Ok(())
}

/// Ensure that templates can be included.
#[test]
fn template_includes() -> Result<()> {
    let app = garden::build::context_from_path("tests/data/garden.yaml")?;
    let config_id = app.get_root_id();
    let context = garden::query::find_tree(&app, config_id, "tree-echo", None)?;
    let config = app.get_root_config();

    let tree = &config.trees[context.tree];
    let result = garden::eval::tree_value(config, "${template-variable}", context.tree, None);
    assert_eq!(result, "template");
    let constant = garden::eval::tree_value(config, "${template-constant}", context.tree, None);
    assert_eq!(constant, "constant");
    assert_eq!(tree.commands.len(), 1);
    assert_eq!(tree.commands[0].get_name(), "echo");
    assert_eq!(
        tree.commands[0].get(0).get_expr(),
        "echo Hello, ${TREE_NAME}"
    );

    // Test a template that uses "extend" on a template defined via an include file.
    // The "tree-echo-extended" uses "extend: tree-echo".
    let context = garden::query::find_tree(&app, config_id, "tree-echo-extended", None)?;
    let tree = &config.trees[context.tree];
    let result = garden::eval::tree_value(config, "${template-variable}", context.tree, None);
    let constant = garden::eval::tree_value(config, "${template-constant}", context.tree, None);
    assert_eq!(result, "extended");
    assert_eq!(constant, "constant");
    assert_eq!(tree.commands.len(), 2);
    assert_eq!(tree.commands[0].get_name(), "echo");
    assert_eq!(
        tree.commands[0].get(0).get_expr(),
        "echo Hello, ${TREE_NAME}"
    );
    assert_eq!(tree.commands[1].get_name(), "echo");
    assert_eq!(tree.commands[1].get(0).get_expr(), "echo extended");

    Ok(())
}
