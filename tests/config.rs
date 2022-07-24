mod common;

use anyhow::Result;

/// Defaults
#[test]
fn config_default() {
    let config = garden::model::Configuration::new();
    assert_eq!("zsh", config.shell);
    assert!(config.verbose == false);
    assert_eq!("", config.root.get_expr());
}

/// Core garden settings
#[test]
fn core() {
    let string = r#"
    garden:
        root: /tmp
    "#
    .to_string();

    let config = common::from_string(&string);
    assert_eq!(std::path::PathBuf::from("/tmp"), config.root_path);
}

/// Variables
#[test]
fn variables() {
    let string = r#"
    garden:
        root: ~/src
    variables:
        foo: foo_value
        bar: ${foo}
    "#
    .to_string();

    let config = common::from_string(&string);
    assert_eq!(3, config.variables.len());

    let mut i = 0;
    assert_eq!("GARDEN_ROOT", config.variables[i].get_name());
    assert_eq!("/home/test/src", config.variables[i].get_expr());
    assert_eq!("/home/test/src", *config.variables[i].get_value().unwrap());
    i += 1;

    assert_eq!("foo", config.variables[i].get_name());
    assert_eq!("foo_value", config.variables[i].get_expr());
    assert_eq!(None, config.variables[i].get_value());
    i += 1;

    assert_eq!("bar", config.variables[i].get_name());
    assert_eq!("${foo}", config.variables[i].get_expr());
    assert_eq!(None, config.variables[i].get_value());
}

/// Commands
#[test]
fn commands() {
    let string = r#"
    commands:
        test_cmd: echo cmd
        test_cmd_vec:
            - echo first
            - echo second
    "#
    .to_string();

    let config = common::from_string(&string);
    assert_eq!(2, config.commands.len());

    assert_eq!("test_cmd", config.commands[0].get_name());
    assert_eq!(1, config.commands[0].len());
    assert_eq!("echo cmd", config.commands[0].get(0).get_expr());

    assert_eq!("test_cmd_vec", config.commands[1].get_name());
    assert_eq!(2, config.commands[1].len());
    assert_eq!("echo first", config.commands[1].get(0).get_expr());
    assert_eq!("echo second", config.commands[1].get(1).get_expr());
}

/// Templates
#[test]
fn templates() {
    let string = r#"
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
    .to_string();

    let config = common::from_string(&string);
    assert_eq!(3, config.templates.len());
    assert_eq!("template1", config.templates[0].get_name());
    assert_eq!(1, config.templates[0].variables.len());
    assert_eq!("foo", config.templates[0].variables[0].get_name());
    assert_eq!("bar", config.templates[0].variables[0].get_expr());

    assert_eq!(2, config.templates[0].environment.len());
    assert_eq!("ENV=", config.templates[0].environment[0].get_name());
    assert_eq!(1, config.templates[0].environment[0].len());
    assert_eq!(
        "${foo}env",
        config.templates[0].environment[0].get(0).get_expr()
    );

    assert_eq!("THEPATH", config.templates[0].environment[1].get_name());
    assert_eq!(2, config.templates[0].environment[1].len());
    assert_eq!(
        "${foo}",
        config.templates[0].environment[1].get(0).get_expr()
    );
    assert_eq!(
        "${ENV}",
        config.templates[0].environment[1].get(1).get_expr()
    );

    assert_eq!("template2", config.templates[1].get_name());
    assert_eq!(vec!["template1"], config.templates[1].extend);
    assert_eq!(3, config.templates[1].variables.len());
    assert_eq!("baz", config.templates[1].variables[0].get_name());
    assert_eq!("zax", config.templates[1].variables[0].get_expr());
    assert_eq!("zee", config.templates[1].variables[1].get_name());
    assert_eq!("${foo}", config.templates[1].variables[1].get_expr());
    assert_eq!("foo", config.templates[1].variables[2].get_name());
    assert_eq!("bar", config.templates[1].variables[2].get_expr());

    assert_eq!("template3", config.templates[2].get_name());
    assert_eq!(vec!["template1", "template2"], config.templates[2].extend);
    assert_eq!(5, config.templates[2].variables.len());
    assert_eq!("foo", config.templates[2].variables[0].get_name());
    assert_eq!("boo", config.templates[2].variables[0].get_expr());
}

/// Groups
#[test]
fn groups() {
    let config = common::garden_config();
    assert!(config.groups.len() >= 2);
    assert_eq!("cola", config.groups[0].get_name());
    assert_eq!(vec!["git", "cola", "python/qtpy"], config.groups[0].members);

    assert_eq!("test", config.groups[1].get_name());
    assert_eq!(vec!["a", "b", "c"], config.groups[1].members);
}

/// Trees
#[test]
fn trees() {
    let config = common::garden_config();
    assert!(config.trees.len() >= 6);

    // git
    let tree0 = &config.trees[0];
    assert!(tree0.environment.is_empty());
    assert_eq!(3, tree0.commands.len());

    assert_eq!("git", tree0.get_name());
    assert_eq!("git", tree0.get_path().get_expr()); // picks up default value
    assert_eq!(vec!["makefile"], tree0.templates);

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
    assert_eq!(vec!["makefile", "python"], tree1.templates);

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

    assert_eq!(4, tree1.commands.len());
    // From the tree
    assert_eq!("build", tree1.commands[0].get_name());
    assert_eq!("install", tree1.commands[1].get_name());
    assert_eq!("test", tree1.commands[2].get_name());
    assert_eq!("test", tree1.commands[3].get_name());
    // From the template
    assert_eq!(1, tree1.commands[2].len());
    assert_eq!("make test", tree1.commands[2].get(0).get_expr());
    // From the tree
    assert_eq!(2, tree1.commands[3].len());
    assert_eq!("git status --short", tree1.commands[3].get(0).get_expr());
    assert_eq!("make tox", tree1.commands[3].get(1).get_expr());

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
}

/// Gardens
#[test]
fn gardens() {
    let config = common::garden_config();
    test_gardens(&config);
}

#[test]
fn gardens_json() {
    let string = r#"
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
    .to_string();

    let config = common::from_string(&string);
    test_gardens(&config);
}

fn test_gardens(config: &garden::model::Configuration) {
    assert!(config.gardens.len() >= 2);

    // "cola" garden
    assert_eq!("cola", config.gardens[0].get_name());

    assert!(config.gardens[0].trees.is_empty());
    assert!(config.gardens[0].gitconfig.is_empty());

    assert_eq!(1, config.gardens[0].groups.len());
    assert_eq!("cola", config.gardens[0].groups[0]);

    assert_eq!(1, config.gardens[0].commands.len());
    assert_eq!("summary", config.gardens[0].commands[0].get_name());
    assert_eq!(2, config.gardens[0].commands[0].len());
    assert_eq!(
        "git branch",
        config.gardens[0].commands[0].get(0).get_expr()
    );
    assert_eq!(
        "git status --short",
        config.gardens[0].commands[0].get(1).get_expr()
    );

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

    assert_eq!(vec!["cola"], config.gardens[1].groups);
    assert_eq!(vec!["gitk"], config.gardens[1].trees);

    assert_eq!(config.gardens[1].gitconfig.len(), 2);
    assert_eq!("user.name", config.gardens[1].gitconfig[0].get_name());
    assert_eq!("A U Thor", config.gardens[1].gitconfig[0].get_expr());
    assert_eq!("user.email", config.gardens[1].gitconfig[1].get_name());
    assert_eq!(
        "author@example.com",
        config.gardens[1].gitconfig[1].get_expr()
    );
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
    let options = garden::build::command_options().verbose(true);
    let app = garden::build::context_from_path("tests/data/garden.yaml", options)?;

    let config = app.get_root_config();
    assert_eq!(2, config.grafts.len());

    assert_eq!("graft", config.grafts[0].get_name());
    let graft_id = config.grafts[0].get_id();
    assert!(graft_id.is_some());
    assert_eq!(2usize, graft_id.unwrap().into());

    assert_eq!("libs", config.grafts[1].get_name());
    let graft_id = config.grafts[1].get_id();
    assert!(graft_id.is_some());
    assert_eq!(5usize, graft_id.unwrap().into());

    Ok(())
}
