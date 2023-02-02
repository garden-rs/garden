pub mod common;

use anyhow::{Context, Result};

use garden::string;

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

    assert!(config.trees.contains_key("tree-zero")); // includes/trees.yaml
    assert!(config.trees.contains_key("tree-echo-nested")); // includes/trees.yaml
    assert!(config.trees.contains_key("tree-echo-extended-tree-inner")); // includes/trees.yaml
    assert!(config.trees.contains_key("tree-echo-extended-tree")); // garden.yaml overrides includes/trees.yaml
    assert!(config.trees.contains_key("example/tree")); // garden.yaml
    assert!(config.trees.contains_key("example/link")); // garden.yaml
    assert!(config.trees.contains_key("link")); // garden.yaml
    assert!(config.trees.contains_key("current")); // garden.yaml
    assert!(config.trees.contains_key("example/shallow")); // garden.yaml
    assert!(config.trees.contains_key("example/single-branch")); // garden.yaml
    assert!(config.trees.contains_key("tree1")); // garden.yaml
    assert!(config.trees.contains_key("tree2")); // garden.yaml
    assert!(config.trees.contains_key("tree-echo-extended")); // garden.yaml

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

    let tree = config.trees.get(&context.tree).context("tree-echo")?;
    let result = garden::eval::tree_value(config, "${template-variable}", &context.tree, None);
    assert_eq!(result, "template");
    let constant = garden::eval::tree_value(config, "${template-constant}", &context.tree, None);
    assert_eq!(constant, "constant");
    assert_eq!(1, tree.commands.len());
    let echo_cmd_opt = tree.commands.get("echo");
    assert!(echo_cmd_opt.is_some());
    let echo_cmd = echo_cmd_opt.context("echo")?;
    assert_eq!(1, echo_cmd.len());
    assert_eq!(echo_cmd[0].get_expr(), "echo Hello, ${TREE_NAME}");

    // Test a template that uses "extend" on a template defined via an include file.
    // The "tree-echo-extended" uses "extend: tree-echo".
    let context = garden::query::find_tree(&app, config_id, "tree-echo-extended", None)?;
    let tree = config
        .trees
        .get(&context.tree)
        .context("tree-echo-extended")?;
    let result = garden::eval::tree_value(config, "${template-variable}", &context.tree, None);
    let constant = garden::eval::tree_value(config, "${template-constant}", &context.tree, None);
    assert_eq!(result, "extended");
    assert_eq!(constant, "constant");
    assert_eq!(tree.commands.len(), 1);

    let echo_cmd_opt = tree.commands.get("echo");
    assert!(echo_cmd_opt.is_some());

    let echo_cmd = echo_cmd_opt.context("echo")?;
    assert_eq!(1, echo_cmd.len());
    assert_eq!(echo_cmd[0].get_expr(), "echo extended");

    // Test a tree that uses "templates" with a template from a nested include file.
    let context = garden::query::find_tree(&app, config_id, "tree-echo-nested", None)?;
    let tree = &config
        .trees
        .get(&context.tree)
        .context("tree-echo-nested")?;
    let result = garden::eval::tree_value(config, "${template-variable}", &context.tree, None);
    let constant = garden::eval::tree_value(config, "${template-constant}", &context.tree, None);
    assert_eq!(constant, "constant");
    assert_eq!(result, "nested");

    let echo_cmd_opt = tree.commands.get("echo");
    assert!(echo_cmd_opt.is_some());

    let echo_cmd = echo_cmd_opt.context("echo")?;
    assert_eq!(1, echo_cmd.len());
    assert_eq!(echo_cmd[0].get_expr(), "echo Hello, ${TREE_NAME}");

    // Test a tree that uses "extend" on a tree defined via an include file.
    let context = garden::query::find_tree(&app, config_id, "tree-echo-extended-tree-inner", None)?;
    let result = garden::eval::tree_value(config, "${template-variable}", &context.tree, None);
    assert_eq!(result, "extended-tree");

    let result = garden::eval::tree_value(config, "${tree-variable}", &context.tree, None);
    assert_eq!(result, "nested");

    let result = garden::eval::tree_value(config, "${tree-override}", &context.tree, None);
    assert_eq!(result, "extended-tree");

    // Test a tree that uses "extend" on a tree defined via an include file.
    // This tree is overridden by the top-level garden.yaml.
    let context = garden::query::find_tree(&app, config_id, "tree-echo-extended-tree", None)?;
    let result = garden::eval::tree_value(config, "${template-variable}", &context.tree, None);
    assert_eq!(result, "top-level");

    let result = garden::eval::tree_value(config, "${tree-override}", &context.tree, None);
    assert_eq!(result, "top-level");

    // "tree-variable" is provided by "tree-echo-nested" via "extend" and is not overriden.
    let result = garden::eval::tree_value(config, "${tree-variable}", &context.tree, None);
    assert_eq!(result, "nested");

    // "extended-variable" is provided by the inner-most "tree-echo-extended-tree".
    // "tree-echo-extended" is sparsely overridden by the top-level garden.yaml.
    // "extended-variable" is not overridden so the inner-most value is retained.
    let result = garden::eval::tree_value(config, "${extended-variable}", &context.tree, None);
    assert_eq!(result, "extended-tree");

    // "replacement-tree" is not sparsely uoverriden -- it is replaced. The variables should
    // evaluate to an empty string because the replacement tree does not define the variable.
    let context = garden::query::find_tree(&app, config_id, "replacement-tree", None)?;
    let result = garden::eval::tree_value(config, "${tree-variable}", &context.tree, None);
    assert_eq!(result, "");

    let replacement_tree = config
        .trees
        .get(&context.tree)
        .context("replacement-tree")?;
    // The replacement tree provides no commands.
    assert!(!replacement_tree.commands.contains_key("tree-command"));
    // The replacement tree replaces the URL for the origin remote.
    assert_eq!(
        "https://example.com/replacement/tree",
        replacement_tree
            .remotes
            .get("origin")
            .context("origin")?
            .get_expr()
    );

    Ok(())
}

/// Ensure that commands are overridden when defined in multiple files.
#[test]
fn command_overrides() -> Result<()> {
    let string = string!(
        r#"
    garden:
      includes: tests/data/includes/commands.yaml
    "#
    );

    // Base case: the "echo" command is read.
    let config = common::from_string(&string);
    assert_eq!(config.commands.len(), 2);
    assert!(config.commands.get("echo").is_some());
    assert!(config.commands.get("test").is_some());

    let echo_cmd = config.commands.get("echo").context("echo")?;
    assert_eq!(1, echo_cmd.len());
    assert_eq!(echo_cmd[0].get_expr(), "echo commands.yaml");

    // If the same command is seen twice it is only defined once.
    let string = string!(
        r#"
    garden:
      includes:
        - tests/data/includes/commands.yaml
        - tests/data/includes/commands.yaml
    "#
    );
    let config = common::from_string(&string);
    assert_eq!(config.commands.len(), 2);

    // If the same command is seen twice the last one wins.
    let string = string!(
        r#"
    garden:
      includes:
        - tests/data/includes/commands.yaml
        - tests/data/includes/commands-override.yaml
    "#
    );
    let config = common::from_string(&string);
    assert_eq!(config.commands.len(), 2);
    assert!(config.commands.get("echo").is_some());
    assert!(config.commands.get("test").is_some());

    let echo_cmd = config.commands.get("echo").context("echo")?;
    assert_eq!(1, echo_cmd.len());
    assert_eq!(echo_cmd[0].get_expr(), "echo commands-override.yaml");

    let test_cmd = config.commands.get("test").context("test")?;
    assert_eq!(1, test_cmd.len());
    assert_eq!(test_cmd[0].get_expr(), "echo override test");

    // If the same command is seen in the garden.yaml then it overrides includes.
    let string = string!(
        r#"
    garden:
      includes:
        - tests/data/includes/commands.yaml
        - tests/data/includes/commands-override.yaml
    commands:
      echo: echo top-level override
    "#
    );
    let config = common::from_string(&string);
    assert_eq!(config.commands.len(), 2);
    assert!(config.commands.get("echo").is_some());
    assert!(config.commands.get("test").is_some());

    let echo_cmd = config.commands.get("echo").context("echo")?;
    assert_eq!(1, echo_cmd.len());
    assert_eq!(echo_cmd[0].get_expr(), "echo top-level override");

    Ok(())
}
