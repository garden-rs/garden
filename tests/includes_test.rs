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

    Ok(())
}
