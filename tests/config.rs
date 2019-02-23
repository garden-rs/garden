extern crate garden;

mod common;


/// Defaults
#[test]
fn config_default() {
    let config = garden::model::Configuration::new();
    assert_eq!(config.shell, "zsh");
    assert_eq!(config.verbose, false);
}


/// Core garden settings
#[test]
fn core() {
    let string = r#"
    garden:
        root: /tmp
    "#.to_string();

    let config = common::from_string(&string);
    assert_eq!(config.root_path, std::path::PathBuf::from("/tmp"));
}

/// Variables
#[test]
fn variables() {
    let string = r#"
    variables:
        foo: foo_value
        bar: ${foo}
    "#.to_string();

    let config = common::from_string(&string);
    assert_eq!(config.variables.len(), 3);

    let mut i = 0;
    assert_eq!(config.variables[i].name, "GARDEN_ROOT");
    assert_eq!(config.variables[i].expr, "/home/test/src");
    assert_eq!(config.variables[i].value, Some("/home/test/src".to_string()));
    i += 1;

    assert_eq!(config.variables[i].name, "foo");
    assert_eq!(config.variables[i].expr, "foo_value");
    assert_eq!(config.variables[i].value, None);
    i += 1;

    assert_eq!(config.variables[i].name, "bar");
    assert_eq!(config.variables[i].expr, "${foo}");
    assert_eq!(config.variables[i].value, None);
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
    "#.to_string();

    let config = common::from_string(&string);
    assert_eq!(config.commands.len(), 2);

    assert_eq!(config.commands[0].name, "test_cmd");
    assert_eq!(config.commands[0].values[0].expr, "echo cmd");

    assert_eq!(config.commands[1].name, "test_cmd_vec");
    assert_eq!(config.commands[1].values[0].expr, "echo first");
    assert_eq!(config.commands[1].values[1].expr, "echo second");
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
    "#.to_string();

    let config = common::from_string(&string);
    assert_eq!(config.templates.len(), 3);
    assert_eq!(config.templates[0].name, "template1");
    assert_eq!(config.templates[0].variables.len(), 1);
    assert_eq!(config.templates[0].variables[0].name, "foo");
    assert_eq!(config.templates[0].variables[0].expr, "bar");

    assert_eq!(config.templates[0].environment.len(), 2);
    assert_eq!(config.templates[0].environment[0].name, "ENV=");
    assert_eq!(config.templates[0].environment[0].values.len(), 1);
    assert_eq!(config.templates[0].environment[0].values[0].expr, "${foo}env");

    assert_eq!(config.templates[0].environment[1].name, "THEPATH");
    assert_eq!(config.templates[0].environment[1].values.len(), 2);
    assert_eq!(config.templates[0].environment[1].values[0].expr, "${foo}");
    assert_eq!(config.templates[0].environment[1].values[1].expr, "${ENV}");

    assert_eq!(config.templates[1].name, "template2");
    assert_eq!(config.templates[1].extend, ["template1"]);
    assert_eq!(config.templates[1].variables.len(), 3);
    assert_eq!(config.templates[1].variables[0].name, "baz");
    assert_eq!(config.templates[1].variables[0].expr, "zax");
    assert_eq!(config.templates[1].variables[1].name, "zee");
    assert_eq!(config.templates[1].variables[1].expr, "${foo}");
    assert_eq!(config.templates[1].variables[2].name, "foo");
    assert_eq!(config.templates[1].variables[2].expr, "bar");

    assert_eq!(config.templates[2].name, "template3");
    assert_eq!(config.templates[2].extend, ["template1", "template2"]);
    assert_eq!(config.templates[2].variables.len(), 5);
    assert_eq!(config.templates[2].variables[0].name, "foo");
    assert_eq!(config.templates[2].variables[0].expr, "boo");
}


/// Groups
#[test]
fn groups() {
    let config = common::garden_config();
    assert!(config.groups.len() >= 2);
    assert_eq!(config.groups[0].name, "cola");
    assert_eq!(config.groups[0].members, ["git", "qtpy", "cola"]);

    assert_eq!(config.groups[1].name, "test");
    assert_eq!(config.groups[1].members, ["a", "b", "c"]);
}

/// Trees
#[test]
fn trees() {
    let config = common::garden_config();
    assert_eq!(config.trees.len(), 4);

    // git
    let ref tree0 = config.trees[0];
    assert!(tree0.environment.is_empty());
    assert_eq!(tree0.commands.len(), 3);

    assert_eq!(tree0.name, "git");
    assert_eq!(tree0.path.expr, "git");  // picks up default value
    assert_eq!(tree0.templates, ["makefile"]);

    assert_eq!(tree0.remotes.len(), 1);
    assert_eq!(tree0.remotes[0].name, "origin");
    assert_eq!(tree0.remotes[0].url, "https://github.com/git/git");

    assert_eq!(tree0.variables.len(), 4);

    // TREE_NAME, highest precedence at position 0
    assert_eq!(tree0.variables[0].name, "TREE_NAME");
    assert_eq!(tree0.variables[0].expr, "git");
    assert_eq!(tree0.variables[0].value.as_ref().unwrap(), "git");

    // TREE_PATH, highest precedence at position 0
    assert_eq!(tree0.variables[1].name, "TREE_PATH");
    assert_eq!(tree0.variables[1].expr, "/home/test/src/git");
    assert_eq!(tree0.variables[1].value.as_ref().unwrap(), "/home/test/src/git");

    assert_eq!(tree0.variables[2].name, "prefix");
    assert_eq!(tree0.variables[2].expr, "~/.local");
    // From the template, effectively "hidden"
    assert_eq!(tree0.variables[3].name, "prefix");
    assert_eq!(tree0.variables[3].expr, "${TREE_PATH}/local");
    // gitconfig
    assert_eq!(tree0.gitconfig.len(), 2);
    assert_eq!(tree0.gitconfig[0].name, "user.name");
    assert_eq!(tree0.gitconfig[0].expr, "A U Thor");
    assert_eq!(tree0.gitconfig[0].value, None);
    assert_eq!(tree0.gitconfig[1].name, "user.email");
    assert_eq!(tree0.gitconfig[1].expr, "author@example.com");
    assert_eq!(tree0.gitconfig[1].value, None);

    // cola
    let ref tree1 = config.trees[1];
    assert!(tree1.gitconfig.is_empty());

    assert_eq!(tree1.name, "cola");
    assert_eq!(tree1.path.expr, "git-cola");
    assert_eq!(tree1.templates, ["makefile", "python"]);

    assert_eq!(tree1.remotes.len(), 2);
    assert_eq!(tree1.remotes[0].name, "origin");
    assert_eq!(tree1.remotes[0].url, "https://github.com/git-cola/git-cola");
    assert_eq!(tree1.remotes[1].name, "davvid");
    assert_eq!(tree1.remotes[1].url, "git@github.com:davvid/git-cola.git");

    assert_eq!(tree1.environment.len(), 3);
    // From "python" template
    assert_eq!(tree1.environment[0].name, "PYTHONPATH");
    assert_eq!(tree1.environment[0].values.len(), 1);
    assert_eq!(tree1.environment[0].values[0].expr, "${TREE_PATH}");
    // From tree
    assert_eq!(tree1.environment[1].name, "PATH");
    assert_eq!(tree1.environment[1].values.len(), 2);
    assert_eq!(tree1.environment[1].values[0].expr, "${TREE_PATH}/bin");
    assert_eq!(tree1.environment[1].values[1].expr, "${prefix}");

    assert_eq!(tree1.environment[2].name, "PYTHONPATH");
    assert_eq!(tree1.environment[2].values.len(), 1);
    assert_eq!(tree1.environment[2].values[0].expr, "${TREE_PATH}");

    assert_eq!(tree1.commands.len(), 4);
    // From the tree
    assert_eq!(tree1.commands[0].name, "build");
    assert_eq!(tree1.commands[1].name, "install");
    assert_eq!(tree1.commands[2].name, "test");
    assert_eq!(tree1.commands[3].name, "test");
    // From the template
    assert_eq!(tree1.commands[2].values.len(), 1);
    assert_eq!(tree1.commands[2].values[0].expr, "make test");
    // From the tree
    assert_eq!(tree1.commands[3].values.len(), 2);
    assert_eq!(tree1.commands[3].values[0].expr, "git status --short");
    assert_eq!(tree1.commands[3].values[1].expr, "make tox");

    // annex
    let ref tree3 = config.trees[3];
    assert_eq!(tree3.name, "annex/data");
    // gitconfig
    assert_eq!(tree3.gitconfig.len(), 1);
    assert_eq!(tree3.gitconfig[0].name, "remote.origin.annex-ignore");
    assert_eq!(tree3.gitconfig[0].expr, "true");
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
                "PATH+": "${prefix}"
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
    "#.to_string();

    let config = common::from_string(&string);
    test_gardens(&config);
}

fn test_gardens(config: &garden::model::Configuration) {
    assert_eq!(config.gardens.len(), 2);

    // "cola" garden
    assert_eq!(config.gardens[0].name, "cola");

    assert!(config.gardens[0].trees.is_empty());
    assert!(config.gardens[0].gitconfig.is_empty());

    assert_eq!(config.gardens[0].groups.len(), 1);
    assert_eq!(config.gardens[0].groups[0], "cola");

    assert_eq!(config.gardens[0].commands.len(), 1);
    assert_eq!(config.gardens[0].commands[0].name, "summary");
    assert_eq!(config.gardens[0].commands[0].values.len(), 2);
    assert_eq!(config.gardens[0].commands[0].values[0].expr,
               "git branch");
    assert_eq!(config.gardens[0].commands[0].values[1].expr,
               "git status --short");

    assert_eq!(config.gardens[0].variables.len(), 1);
    assert_eq!(config.gardens[0].variables[0].name, "prefix");
    assert_eq!(config.gardens[0].variables[0].expr,
               "~/apps/git-cola/current");

    assert_eq!(config.gardens[0].environment.len(), 2);
    assert_eq!(config.gardens[0].environment[0].name, "GIT_COLA_TRACE=");
    assert_eq!(config.gardens[0].environment[0].values.len(), 1);
    assert_eq!(config.gardens[0].environment[0].values[0].expr, "full");

    assert_eq!(config.gardens[0].environment[1].name, "PATH+");
    assert_eq!(config.gardens[0].environment[1].values.len(), 1);
    assert_eq!(config.gardens[0].environment[1].values[0].expr, "${prefix}");

    // "git" garden
    assert_eq!(config.gardens[1].name, "git");

    assert!(config.gardens[1].environment.is_empty());
    assert!(config.gardens[1].variables.is_empty());
    assert!(config.gardens[1].commands.is_empty());

    assert_eq!(config.gardens[1].groups, ["cola"]);
    assert_eq!(config.gardens[1].trees, ["gitk"]);

    assert_eq!(config.gardens[1].gitconfig.len(), 2);
    assert_eq!(config.gardens[1].gitconfig[0].name, "user.name");
    assert_eq!(config.gardens[1].gitconfig[0].expr, "A U Thor");
    assert_eq!(config.gardens[1].gitconfig[1].name, "user.email");
    assert_eq!(config.gardens[1].gitconfig[1].expr, "author@example.com");
}


#[test]
fn tree_path() {
    let config = common::garden_config();
    assert!(config.trees.len() >= 3);

    assert_eq!(config.trees[0].path.value.as_ref().unwrap().to_string(),
               "/home/test/src/git");

    // cola is in the "git-cola" subdirectory
    assert_eq!(config.trees[1].path.value.as_ref().unwrap().to_string(),
               "/home/test/src/git-cola");

    // tmp is in "/tmp"
    assert_eq!(config.trees[2].path.value.as_ref().unwrap().to_string(),
               "/tmp");
}
