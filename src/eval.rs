extern crate shellexpand;
extern crate subprocess;
extern crate dirs;

use super::model;


/// Expand variables across all scopes (garden, tree, and global)
fn expand_tree_vars(
    config: &mut model::Configuration,
    tree_idx: model::TreeIndex,
    garden_idx: Option<model::GardenIndex>,
    name: &str,
) -> Result<Option<String>, String> {

    let mut var_idx: usize = 0;
    let mut found = false;

    // First check for the variable at the garden scope.
    // Garden scope overrides tree and global scope.
    if garden_idx.is_some() {
        for (idx, var) in
        config.gardens[garden_idx.unwrap()].variables.iter().enumerate() {
            if var.name == name {
                if var.value.is_some() {
                    return Ok(Some(var.value.as_ref().unwrap().to_string()));
                }
                var_idx = idx;
                found = true;
                break;
            }
        }

        if found {
            let expr =
                config
                .gardens[garden_idx.unwrap()]
                .variables[var_idx]
                .expr.to_string();

            let result = tree_value(config, expr, tree_idx, garden_idx);
            config
                .gardens[garden_idx.unwrap()]
                .variables[var_idx]
                .value = Some(result.to_string());
            return Ok(Some(result.to_string()));
        }
    }

    // Nothing was found -- check for the variable in tree scope.
    found = false;
    var_idx = 0;

    for (idx, var) in config.trees[tree_idx].variables.iter().enumerate() {
        if var.name == name {
            if var.value.is_some() {
                return Ok(Some(var.value.as_ref().unwrap().to_string()));
            }
            found = true;
            var_idx = idx;
            break;
        }
    }

    if found {
        let expr = config.trees[tree_idx].variables[var_idx].expr.to_string();
        let result = tree_value(config, expr, tree_idx, garden_idx);
        config
            .trees[tree_idx]
            .variables[var_idx]
            .value = Some(result.to_string());
        return Ok(Some(result));
    }

    // Nothing was found.  Check for the variable in global/config scope.
    found = false;
    var_idx = 0;

    for (idx, var) in config.variables.iter().enumerate() {
        if var.name == name {
            if var.value.is_some() {
                return Ok(Some(var.value.as_ref().unwrap().to_string()));
            }
            found = true;
            var_idx = idx;
            break;
        }
    }

    if found {
        let expr = config.variables[var_idx].expr.to_string();
        let result = tree_value(config, expr, tree_idx, garden_idx);
        config.variables[var_idx].value = Some(result.to_string());

        return Ok(Some(result));
    }

    // Nothing was found -> empty value
    return Ok(Some("".to_string()));
}


/// Expand variables at global scope only
fn expand_vars(
    config: &mut model::Configuration,
    name: &str,
) -> Result<Option<String>, String> {

    let mut var_idx: usize = 0;
    let mut found = false;

    for (idx, var) in config.variables.iter().enumerate() {
        if var.name == name {
            if var.value.is_some() {
                return Ok(Some(var.value.as_ref().unwrap().to_string()));
            }
            var_idx = idx;
            found = true;
            break;
        }
    }

    if found {
        let expr = config.variables[var_idx].expr.to_string();
        let result = value(config, expr);
        config.variables[var_idx].value = Some(result.to_string());

        return Ok(Some(result));
    }

    // Nothing was found -> empty value
    return Ok(Some("".to_string()));
}


/// Resolve ~ to the current user's home directory
fn home_dir() -> Option<std::path::PathBuf> {
    // Honor $HOME when set in the environment.
    if let Ok(home) = std::env::var("HOME") {
        return Some(std::path::PathBuf::from(home));
    }
    return dirs::home_dir();
}


/// Resolve a variable in a garden/tree/global scope
pub fn tree_value<S: Into<String>>(
    config: &mut model::Configuration,
    expr: S,
    tree_idx: model::TreeIndex,
    garden_idx: Option<model::GardenIndex>,
) -> String {
    let expr_str: String = expr.into();

    let expanded = shellexpand::full_with_context(
        &expr_str, home_dir,
        |x| { return expand_tree_vars(config, tree_idx, garden_idx, x); }
        ).unwrap().to_string();

    return exec_expression(expanded);
}


/// Resolve a variable in configuration/global scope
pub fn value<S: Into<String>>(
    config: &mut model::Configuration,
    expr: S,
) -> String {
    let expr_str: String = expr.into();

    let expanded = shellexpand::full_with_context(
        &expr_str, home_dir,
        |x| { return expand_vars(config, x); }
        ).unwrap().to_string();

    return exec_expression(expanded);
}


/// Evaluate "$ <command>" command strings, AKA "exec expressions".
/// The result of the expression is the stdout output from the command.
pub fn exec_expression(string: String) -> String {
    if string.starts_with("$ ") {
        let mut cmd = string.to_string();
        cmd.remove(0);
        cmd.remove(0);

        let capture = subprocess::Exec::shell(cmd)
            .stdout(subprocess::Redirection::Pipe)
            .capture();
        if let Ok(x) = capture {
            return x.stdout_str().trim_end().to_string();
        }
        // An error occurred running the command -- empty output by design
        return "".to_string();
    }

    return string;
}
