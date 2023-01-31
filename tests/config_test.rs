pub mod common;

use anyhow::{Context, Result};
use indexmap::indexset;

use garden::string;

/// Defaults
#[test]
fn config_default() {
    let config = garden::model::Configuration::new();
    assert!(matches!(config.shell.as_str(), "bash" | "sh" | "zsh"));
    assert_eq!(0, config.verbose);
    assert_eq!("", config.root.get_expr());
}

/// Core garden settings
#[test]
fn core() {
    let string = string!(
        r#"
    garden:
        root: /usr
    "#
    );
    let config = common::from_string(&string);
    assert_eq!(std::path::PathBuf::from("/usr"), config.root_path);
}

/// Variables
#[test]
fn variables() -> Result<()> {
    let string = string!(
        r#"
    garden:
        root: ~/src
    variables:
        foo: foo_value
        bar: ${foo}
    "#
    );
    let config = common::from_string(&string);
    assert_eq!(3, config.variables.len());

    let root_var = config.variables.get("GARDEN_ROOT").context("GARDEN_ROOT")?;
    assert_eq!("/home/test/src", root_var.get_expr());
    assert_eq!(
        Some("/home/test/src"),
        root_var.get_value().map(|x| x.as_str())
    );

    let foo_var = config.variables.get("foo").context("foo")?;
    assert_eq!("foo_value", foo_var.get_expr());
    assert_eq!(None, foo_var.get_value());

    let bar_var = config.variables.get("bar").context("bar")?;
    assert_eq!("${foo}", bar_var.get_expr());
    assert_eq!(None, bar_var.get_value());

    Ok(())
}

/// Commands
#[test]
fn commands() -> Result<()> {
    let string = string!(
        r#"
    commands:
        test_cmd: echo cmd
        test_cmd_vec:
            - echo first
            - echo second
    "#
    );
    let config = common::from_string(&string);
    assert_eq!(2, config.commands.len());

    assert!(config.commands.get("test_cmd").is_some());
    assert_eq!(
        1,
        config
            .commands
            .get("test_cmd")
            .context("test_cmd command")?
            .len()
    );
    assert_eq!(
        "echo cmd",
        config
            .commands
            .get("test_cmd")
            .context("test_cmd command")?
            .get(0)
            .context("test_cmd[0]")?
            .get_expr()
    );

    let test_cmd_vec_opt = config.commands.get("test_cmd_vec");
    assert!(test_cmd_vec_opt.is_some());

    let test_cmd_vec = test_cmd_vec_opt.context("test_cmd_vec")?;
    assert_eq!(2, test_cmd_vec.len());
    assert_eq!("echo first", test_cmd_vec[0].get_expr());
    assert_eq!("echo second", test_cmd_vec[1].get_expr());

    Ok(())
}

/// Templates
#[test]
fn templates() {
    let string = string!(
        r#"
    templates:
        template1:
            variables:
                foo: bar
            environment:
                ENV=: ${foo}env
                THEPATH:
                    - ${foo}
                    - ${ENV}
        template2:
            extend: template1
            variables:
                baz: zax
                zee: ${foo}
        template3:
            extend: [template1, template2]
            variables:
                foo: boo
    "#
    );
    let config = common::from_string(&string);
    assert_eq!(3, config.templates.len());

    let template1 = config.templates.get("template1").unwrap();
    assert_eq!("template1", template1.get_name());
    assert_eq!(1, template1.tree.variables.len());
    assert_eq!("foo", template1.tree.variables[0].get_name());
    assert_eq!("bar", template1.tree.variables[0].get_expr());
    assert_eq!(2, template1.tree.environment.len());
    assert_eq!("ENV=", template1.tree.environment[0].get_name());
    assert_eq!(1, template1.tree.environment[0].len());
    assert_eq!("${foo}env", template1.tree.environment[0].get(0).get_expr());
    assert_eq!("THEPATH", template1.tree.environment[1].get_name());
    assert_eq!(2, template1.tree.environment[1].len());
    assert_eq!("${foo}", template1.tree.environment[1].get(0).get_expr());
    assert_eq!("${ENV}", template1.tree.environment[1].get(1).get_expr());

    let template2 = config.templates.get("template2").unwrap();
    assert_eq!("template2", template2.get_name());
    assert_eq!(indexset! {string!("template1")}, template2.extend);
    assert_eq!(3, template2.tree.variables.len());
    assert_eq!("baz", template2.tree.variables[0].get_name());
    assert_eq!("zax", template2.tree.variables[0].get_expr());
    assert_eq!("zee", template2.tree.variables[1].get_name());
    assert_eq!("${foo}", template2.tree.variables[1].get_expr());
    assert_eq!("foo", template2.tree.variables[2].get_name());
    assert_eq!("bar", template2.tree.variables[2].get_expr());

    let template3 = config.templates.get("template3").unwrap();
    assert_eq!("template3", template3.get_name());
    assert_eq!(
        indexset! {string!("template1"), string!("template2")},
        template3.extend
    );
    assert_eq!(5, template3.tree.variables.len());
    assert_eq!("foo", template3.tree.variables[0].get_name());
    assert_eq!("boo", template3.tree.variables[0].get_expr());
}

/// Groups
#[test]
fn groups() -> Result<()> {
    let config = common::garden_config();
    assert!(config.groups.len() >= 2);
    assert!(config.groups.get("cola").is_some());
    assert_eq!(
        "cola",
        config
            .groups
            .get("cola")
            .context("missing cola group")?
            .get_name()
    );
    assert_eq!(
        indexset! {
            string!("git"),
            string!("cola"),
            string!("python/qtpy")
        },
        config.groups["cola"].members
    );

    let test_group = config.groups.get("test").context("missing text group")?;
    assert_eq!("test", test_group.get_name());
    assert_eq!(
        indexset! {
            string!("a"),
            string!("b"),
            string!("c")
        },
        test_group.members
    );

    Ok(())
}

/// Trees
#[test]
fn trees() -> Result<()> {
    let config = common::garden_config();
    assert!(config.trees.len() >= 6);
    // git
    let tree0 = &config.trees[0];
    assert!(tree0.environment.is_empty());
    assert_eq!(3, tree0.commands.len());

    assert_eq!("git", tree0.get_name());
    assert_eq!("git", tree0.get_path().get_expr()); // picks up default value
    assert_eq!(indexset! {string!("makefile")}, tree0.templates);

    assert_eq!(1, tree0.remotes.len());
    assert_eq!("origin", tree0.remotes[0].get_name());
    assert_eq!("https://github.com/git/git", tree0.remotes[0].get_expr());

    assert_eq!(4, tree0.variables.len());

    // TREE_NAME, highest precedence at position 0
    assert_eq!("TREE_NAME", tree0.variables[0].get_name());
    assert_eq!("git", tree0.variables[0].get_expr());
    assert_eq!("git", tree0.variables[0].get_value().unwrap());

    // TREE_PATH, highest precedence at position 0
    assert_eq!("TREE_PATH", tree0.variables[1].get_name());
    assert_eq!("/home/test/src/git", tree0.variables[1].get_expr());
    assert_eq!(
        "/home/test/src/git",
        tree0.variables[1].get_value().unwrap()
    );

    assert_eq!("prefix", tree0.variables[2].get_name());
    assert_eq!("~/.local", tree0.variables[2].get_expr());
    // From the template, effectively "hidden"
    assert_eq!("prefix", tree0.variables[3].get_name());
    assert_eq!("${TREE_PATH}/local", tree0.variables[3].get_expr());
    // gitconfig
    assert_eq!(2, tree0.gitconfig.len());
    assert_eq!("user.name", tree0.gitconfig[0].get_name());
    assert_eq!("A U Thor", tree0.gitconfig[0].get_expr());
    assert_eq!(None, tree0.gitconfig[0].get_value());
    assert_eq!("user.email", tree0.gitconfig[1].get_name());
    assert_eq!("author@example.com", tree0.gitconfig[1].get_expr());
    assert_eq!(None, tree0.gitconfig[1].get_value());

    // cola
    let tree1 = &config.trees[1];
    assert!(tree1.gitconfig.is_empty());

    assert_eq!("cola", tree1.get_name());
    assert_eq!("git-cola", tree1.get_path().get_expr());
    assert_eq!(
        indexset! {string!("makefile"), string!("python")},
        tree1.templates
    );

    assert_eq!(2, tree1.remotes.len());
    assert_eq!("origin", tree1.remotes[0].get_name());
    assert_eq!(
        "https://github.com/git-cola/git-cola",
        tree1.remotes[0].get_expr()
    );
    assert_eq!("davvid", tree1.remotes[1].get_name());
    assert_eq!(
        "git@github.com:davvid/git-cola.git",
        tree1.remotes[1].get_expr()
    );

    assert_eq!(3, tree1.environment.len());
    // From "python" template
    assert_eq!("PYTHONPATH", tree1.environment[0].get_name());
    assert_eq!(1, tree1.environment[0].len());
    assert_eq!("${TREE_PATH}", tree1.environment[0].get(0).get_expr());
    // From tree
    assert_eq!("PATH", tree1.environment[1].get_name());
    assert_eq!(2, tree1.environment[1].len());
    assert_eq!("${prefix}/bin", tree1.environment[1].get(0).get_expr());
    assert_eq!("${TREE_PATH}/bin", tree1.environment[1].get(1).get_expr());

    assert_eq!("PYTHONPATH", tree1.environment[2].get_name());
    assert_eq!(1, tree1.environment[2].len());
    assert_eq!(
        "${GARDEN_ROOT}/python/send2trash",
        tree1.environment[2].get(0).get_expr()
    );

    assert_eq!(3, tree1.commands.len());
    // From the tree
    assert!(tree1.commands.get("build").is_some());
    assert!(tree1.commands.get("install").is_some());
    assert!(tree1.commands.get("test").is_some());
    // From the template
    let test_cmd = tree1.commands.get("test").context("test")?;
    assert_eq!(2, test_cmd.len());
    // Commands from the tree override commands defined in the base template.
    assert_eq!("git status --short", test_cmd[0].get_expr());
    assert_eq!("make tox", test_cmd[1].get_expr());

    // annex/data
    let tree3 = &config.trees[4];
    assert_eq!("annex/data", tree3.get_name());
    // gitconfig
    assert_eq!(1, tree3.gitconfig.len());
    assert_eq!("remote.origin.annex-ignore", tree3.gitconfig[0].get_name());
    assert_eq!("true", tree3.gitconfig[0].get_expr());
    // remotes
    assert_eq!(2, tree3.remotes.len());
    assert_eq!("origin", tree3.remotes[0].get_name());
    assert_eq!(
        "git@example.com:git-annex/data.git",
        tree3.remotes[0].get_expr()
    );
    assert_eq!("local", tree3.remotes[1].get_name());
    assert_eq!("${GARDEN_ROOT}/annex/local", tree3.remotes[1].get_expr());

    // annex/local extends annex/data
    let tree4 = &config.trees[5];
    assert_eq!("annex/local", tree4.get_name());
    // gitconfig
    assert_eq!(1, tree4.gitconfig.len());
    assert_eq!("remote.origin.annex-ignore", tree4.gitconfig[0].get_name());
    assert_eq!("true", tree4.gitconfig[0].get_expr());
    // remotes
    assert_eq!(1, tree4.remotes.len());
    assert_eq!("origin", tree4.remotes[0].get_name());
    assert_eq!(
        "git@example.com:git-annex/data.git",
        tree4.remotes[0].get_expr()
    );

    Ok(())
}

/// Gardens
#[test]
fn gardens() -> Result<()> {
    let config = common::garden_config();
    test_gardens(&config)
}

#[test]
fn gardens_json() -> Result<()> {
    let string = string!(
        r#"
{
    "gardens": {
        "cola": {
            "groups": "cola",
            "variables": {
                "prefix": "~/apps/git-cola/current"
            },
            "environment": {
                "GIT_COLA_TRACE=": "full",
                "PATH+": "${prefix}/bin"
            },
            "commands": {
                "summary": [
                    "git branch",
                    "git status --short"
                ]
            }
        },
        "git": {
            "groups": "cola",
            "trees": "gitk",
            "gitconfig": {
                "user.name": "A U Thor",
                "user.email": "author@example.com"
            }
        }
    }
}
    "#
    );
    let config = common::from_string(&string);
    test_gardens(&config)
}

fn test_gardens(config: &garden::model::Configuration) -> Result<()> {
    assert!(config.gardens.len() >= 2);

    // "cola" garden
    assert_eq!("cola", config.gardens[0].get_name());

    assert!(config.gardens[0].trees.is_empty());
    assert!(config.gardens[0].gitconfig.is_empty());

    assert_eq!(1, config.gardens[0].groups.len());
    assert_eq!("cola", config.gardens[0].groups[0]);

    assert_eq!(1, config.gardens[0].commands.len());
    let summary_cmd_opt = config.gardens[0].commands.get("summary");
    assert!(summary_cmd_opt.is_some());

    let summary_cmd = summary_cmd_opt.context("summary")?;
    assert_eq!(2, summary_cmd.len());
    assert_eq!("git branch", summary_cmd[0].get_expr());
    assert_eq!("git status --short", summary_cmd[1].get_expr());

    assert_eq!(1, config.gardens[0].variables.len());
    assert_eq!("prefix", config.gardens[0].variables[0].get_name());
    assert_eq!(
        "~/apps/git-cola/current",
        config.gardens[0].variables[0].get_expr()
    );

    assert_eq!(2, config.gardens[0].environment.len());
    assert_eq!(
        "GIT_COLA_TRACE=",
        config.gardens[0].environment[0].get_name()
    );
    assert_eq!(1, config.gardens[0].environment[0].len());
    assert_eq!("full", config.gardens[0].environment[0].get(0).get_expr());

    assert_eq!("PATH+", config.gardens[0].environment[1].get_name());
    assert_eq!(1, config.gardens[0].environment[1].len());
    assert_eq!(
        "${prefix}/bin",
        config.gardens[0].environment[1].get(0).get_expr()
    );

    // "git" garden
    assert_eq!("git", config.gardens[1].get_name());

    assert!(config.gardens[1].environment.is_empty());
    assert!(config.gardens[1].variables.is_empty());
    assert!(config.gardens[1].commands.is_empty());

    assert_eq!(indexset! {string!("cola")}, config.gardens[1].groups);
    assert_eq!(indexset! {string!("gitk")}, config.gardens[1].trees);

    assert_eq!(config.gardens[1].gitconfig.len(), 2);
    assert_eq!("user.name", config.gardens[1].gitconfig[0].get_name());
    assert_eq!("A U Thor", config.gardens[1].gitconfig[0].get_expr());
    assert_eq!("user.email", config.gardens[1].gitconfig[1].get_name());
    assert_eq!(
        "author@example.com",
        config.gardens[1].gitconfig[1].get_expr()
    );

    Ok(())
}

#[test]
fn tree_path() {
    let config = common::garden_config();
    assert!(config.trees.len() >= 4);

    assert_eq!(
        "/home/test/src/git",
        *config.trees[0].path_as_ref().unwrap()
    );
    // cola is in the "git-cola" subdirectory
    assert_eq!(
        "/home/test/src/git-cola",
        *config.trees[1].path_as_ref().unwrap()
    );
    assert_eq!(
        "/home/test/src/python/qtpy",
        *config.trees[2].path_as_ref().unwrap()
    );
}

#[test]
fn test_template_url() {
    let config = common::garden_config();
    assert!(config.trees.len() > 3);
    // The "tmp" tree uses the "local" template which defines a URL.
    let tree = &config.trees[3];
    assert_eq!("tmp", tree.get_name());
    assert_eq!(1, tree.remotes.len());
    assert_eq!("origin", tree.remotes[0].get_name());
    assert_eq!("${local}/${TREE_NAME}", tree.remotes[0].get_expr());
}

#[test]
fn read_grafts() -> Result<()> {
    let app = garden::build::context_from_path("tests/data/garden.yaml")?;
    let config = app.get_root_config();
    assert_eq!(2, config.grafts.len());

    assert_eq!("graft", config.grafts[0].get_name());
    let graft_id = config.grafts[0].get_id();
    assert!(graft_id.is_some());

    let graft_node_id: usize = graft_id.unwrap().into();
    assert_eq!(2usize, graft_node_id);

    assert_eq!("libs", config.grafts[1].get_name());
    let graft_id = config.grafts[1].get_id();
    assert!(graft_id.is_some());

    let graft_node_id: usize = graft_id.unwrap().into();
    assert_eq!(5usize, graft_node_id);

    Ok(())
}
