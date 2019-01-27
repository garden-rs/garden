extern crate garden;


fn from_yaml_string(string: &String) -> garden::model::Configuration {
    let mut config = garden::model::Configuration::new();
    let file_format = garden::config::FileFormat::YAML;
    garden::config::parse(string, file_format, false, &mut config);

    return config;
}


/// Test defaults
fn config_default() {
    let config = garden::model::Configuration::new();
    assert_eq!(config.shell.to_string_lossy(), "zsh");
    assert_eq!(config.environment_variables, true);
    assert_eq!(config.verbose, false);
}


/// Test core garden settings
#[test]
fn core() {
    let string = r#"
garden:
    root: /tmp
    environment_variables: false
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.root_path, std::path::PathBuf::from("/tmp"));
    assert_eq!(config.environment_variables, false);
}

/// Test variables
#[test]
fn variables() {
    let string = r#"
variables:
    foo: foo_value
    bar: ${foo}
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.variables.len(), 2);

    assert_eq!(config.variables[0].name, "foo");
    assert_eq!(config.variables[0].var.expr, "foo_value");
    assert_eq!(config.variables[0].var.value, None);

    assert_eq!(config.variables[1].name, "bar");
    assert_eq!(config.variables[1].var.expr, "${foo}");
    assert_eq!(config.variables[1].var.value, None);
}

/// Test commands
#[test]
fn commands() {
    let string = r#"
commands:
    test_cmd: echo cmd
    test_cmd_vec:
        - echo first
        - echo second
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.commands.len(), 2);

    assert_eq!(config.commands[0].name, "test_cmd");
    assert_eq!(config.commands[0].values[0].expr, "echo cmd");

    assert_eq!(config.commands[1].name, "test_cmd_vec");
    assert_eq!(config.commands[1].values[0].expr, "echo first");
    assert_eq!(config.commands[1].values[1].expr, "echo second");
}
