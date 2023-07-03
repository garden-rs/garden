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
fn core() -> Result<()> {
    let string = string!(
        r#"
    garden:
        root: /usr
    "#
    );
    let app_context = common::garden_context_from_string(&string)?;
    let config = app_context.get_root_config();
    assert_eq!(std::path::PathBuf::from("/usr"), config.root_path);

    Ok(())
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
    let app_context = common::garden_context_from_string(&string)?;
    let config = app_context.get_root_config();
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
    let app_context = common::garden_context_from_string(&string)?;
    let config = app_context.get_root_config();
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
fn templates() -> Result<()> {
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
    let app_context = common::garden_context_from_string(&string)?;
    let config = app_context.get_root_config();
    assert_eq!(3, config.templates.len());

    let template1 = config.templates.get("template1").unwrap();
    assert_eq!("template1", template1.get_name());
    assert_eq!(1, template1.tree.variables.len());

    let foo_var = template1.tree.variables.get("foo").context("foo")?;
    assert_eq!("bar", foo_var.get_expr());

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

    let baz_var = template2.tree.variables.get("baz").context("baz")?;
    assert_eq!("zax", baz_var.get_expr());

    let zee_var = template2.tree.variables.get("zee").context("zee")?;
    assert_eq!("${foo}", zee_var.get_expr());

    let foo_var = template2.tree.variables.get("foo").context("foo")?;
    assert_eq!("bar", foo_var.get_expr());

    let template3 = config.templates.get("template3").unwrap();
    assert_eq!("template3", template3.get_name());
    assert_eq!(
        indexset! {string!("template1"), string!("template2")},
        template3.extend
    );
    assert_eq!(3, template3.tree.variables.len());

    let foo_var = template3.tree.variables.get("foo").context("foo")?;
    assert_eq!("boo", foo_var.get_expr());

    Ok(())
}

/// Groups
#[test]
fn groups() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
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
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    assert!(config.trees.len() >= 6);
    // git
    let tree0 = &config.trees[0];
    assert!(tree0.environment.is_empty());
    assert_eq!(3, tree0.commands.len());

    assert_eq!("git", tree0.get_name());
    assert_eq!("git", tree0.get_path().get_expr()); // picks up default value
    assert_eq!(indexset! {string!("makefile")}, tree0.templates);

    assert_eq!(1, tree0.remotes.len());

    let origin_var = tree0.remotes.get("origin").context("origin")?;
    assert_eq!("https://github.com/git/git", origin_var.get_expr());

    assert_eq!(3, tree0.variables.len());

    // TREE_NAME, highest precedence.
    let tree_name_var = tree0.variables.get("TREE_NAME").context("TREE_NAME")?;
    assert_eq!("git", tree_name_var.get_expr());
    assert_eq!("git", tree_name_var.get_value().unwrap());

    // TREE_PATH, highest precedence.
    let tree_path_var = tree0.variables.get("TREE_PATH").context("TREE_PATH")?;
    assert_eq!("/home/test/src/git", tree_path_var.get_expr());
    assert_eq!("/home/test/src/git", tree_path_var.get_value().unwrap());

    let prefix_var = tree0.variables.get("prefix").context("prefix")?;
    assert_eq!("~/.local", prefix_var.get_expr());
    // From the template, effectively "hidden"
    // assert_eq!("${TREE_PATH}/local", prefix_var.get_expr());

    // gitconfig
    assert_eq!(2, tree0.gitconfig.len());
    let user_name_var = tree0.gitconfig.get("user.name").context("user.name")?;
    assert_eq!(
        "A U Thor",
        user_name_var.get(0).context("user.name expr")?.get_expr()
    );
    assert_eq!(
        None,
        user_name_var
            .get(0)
            .context("None for user.name value")?
            .get_value()
    );
    let user_email_var = tree0.gitconfig.get("user.email").context("user.email")?;
    assert_eq!(
        "author@example.com",
        user_email_var.get(0).context("user.email expr")?.get_expr()
    );
    assert_eq!(
        None,
        user_email_var
            .get(0)
            .context("user.email value")?
            .get_value()
    );

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

    let origin_var = tree1.remotes.get("origin").context("origin")?;
    assert_eq!(
        "https://github.com/git-cola/git-cola",
        origin_var.get_expr()
    );

    let remote_var = tree1.remotes.get("davvid").context("davvid")?;
    assert_eq!("git@github.com:davvid/git-cola.git", remote_var.get_expr());

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
    let annex_ignore_var = tree3
        .gitconfig
        .get("remote.origin.annex-ignore")
        .context("annex-ignore")?;
    assert_eq!(
        "true",
        annex_ignore_var
            .get(0)
            .context("remote.origin.annex-ignore expr")?
            .get_expr()
    );
    // remotes
    assert_eq!(2, tree3.remotes.len());
    let origin_var = tree3.remotes.get("origin").context("origin")?;
    assert_eq!("git@example.com:git-annex/data.git", origin_var.get_expr());
    let remote_var = tree3.remotes.get("local").context("local")?;
    assert_eq!("${GARDEN_ROOT}/annex/local", remote_var.get_expr());

    // annex/local extends annex/data
    let tree4 = &config.trees[5];
    assert_eq!("annex/local", tree4.get_name());
    // gitconfig
    assert_eq!(1, tree4.gitconfig.len());
    let annex_ignore_var = tree4
        .gitconfig
        .get("remote.origin.annex-ignore")
        .context("annex-ignore")?;
    assert_eq!(
        "true",
        annex_ignore_var
            .get(0)
            .context("annex-ignore expr")?
            .get_expr()
    );
    // remotes
    assert_eq!(2, tree4.remotes.len());
    let origin_var = tree4.remotes.get("origin").context("origin")?;
    assert_eq!("git@example.com:git-annex/data.git", origin_var.get_expr());
    let remote_var = tree4.remotes.get("local").context("local")?;
    assert_eq!("${GARDEN_ROOT}/annex/local", remote_var.get_expr());

    Ok(())
}

/// Gardens
#[test]
fn gardens() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
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
    let app_context = common::garden_context_from_string(&string)?;
    let config = app_context.get_root_config();
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

    let prefix_var = config.gardens[0]
        .variables
        .get("prefix")
        .context("prefix")?;
    assert_eq!("~/apps/git-cola/current", prefix_var.get_expr());

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
    let user_name_var = config.gardens[1]
        .gitconfig
        .get("user.name")
        .context("user.name")?;
    assert_eq!(
        "A U Thor",
        user_name_var.get(0).context("user.name expr")?.get_expr()
    );

    let user_email_var = config.gardens[1]
        .gitconfig
        .get("user.email")
        .context("user.email")?;
    assert_eq!(
        "author@example.com",
        user_email_var.get(0).context("user.email expr")?.get_expr()
    );

    Ok(())
}

#[test]
fn tree_path() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
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

    Ok(())
}

#[test]
fn test_template_url() -> Result<()> {
    let app_context = common::garden_context()?;
    let config = app_context.get_root_config();
    assert!(config.trees.len() > 3);
    // The "tmp" tree uses the "local" template which defines a URL.
    let tree = &config.trees[3];
    assert_eq!("tmp", tree.get_name());
    assert_eq!(1, tree.remotes.len());
    let origin_var = tree.remotes.get("origin").context("origin")?;
    assert_eq!("${local}/${TREE_NAME}", origin_var.get_expr());

    Ok(())
}

#[test]
fn read_grafts() -> Result<()> {
    let app = garden::model::ApplicationContext::from_path_string("tests/data/garden.yaml")?;
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
