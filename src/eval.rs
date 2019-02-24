extern crate dirs;
extern crate glob;
extern crate shellexpand;
extern crate subprocess;

use std::collections::HashMap;

use ::model;
use ::syntax;


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

            let result = tree_value(config, &expr, tree_idx, garden_idx);
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
        let result = tree_value(config, &expr, tree_idx, garden_idx);
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
        let result = tree_value(config, &expr, tree_idx, garden_idx);
        config.variables[var_idx].value = Some(result.to_string());

        return Ok(Some(result));
    }

    // If nothing was found then check for environment variables.
    if let Ok(env_value) = std::env::var(name) {
        return Ok(Some(env_value.to_string()));
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
        let result = value(config, &expr);
        config.variables[var_idx].value = Some(result.to_string());

        return Ok(Some(result));
    }

    // If nothing was found then check for environment variables.
    if let Ok(env_value) = std::env::var(name) {
        return Ok(Some(env_value.to_string()));
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
pub fn tree_value(
    config: &mut model::Configuration,
    expr: &str,
    tree_idx: model::TreeIndex,
    garden_idx: Option<model::GardenIndex>,
) -> String {
    let expanded = shellexpand::full_with_context(
        expr, home_dir,
        |x| { return expand_tree_vars(config, tree_idx, garden_idx, x); }
        ).unwrap().to_string();

    return exec_expression(&expanded);
}


/// Resolve a variable in configuration/global scope
pub fn value(
    config: &mut model::Configuration,
    expr: &str,
) -> String {
    let expanded = shellexpand::full_with_context(
        expr, home_dir,
        |x| { return expand_vars(config, x); }
        ).unwrap().to_string();

    return exec_expression(&expanded);
}


/// Evaluate "$ <command>" command strings, AKA "exec expressions".
/// The result of the expression is the stdout output from the command.
pub fn exec_expression(string: &String) -> String {
    if syntax::is_exec(&string) {
        let cmd = syntax::trim_exec(&string);
        let capture = subprocess::Exec::shell(cmd)
            .stdout(subprocess::Redirection::Pipe)
            .capture();
        if let Ok(x) = capture {
            return x.stdout_str().trim_end().to_string();
        }
        // An error occurred running the command -- empty output by design
        return "".to_string();
    }

    string.to_string()
}


/// Evaluate a variable in the given context
pub fn multi_variable(
    config: &mut model::Configuration,
    multi_var: &mut model::MultiVariable,
    context: &model::TreeContext,
) -> Vec<String> {

    let mut result = Vec::new();

    for var in &multi_var.values {
        if let Some(ref value) = var.value {
            result.push(value.to_string());
            continue;
        }

        let mut value = tree_value(
            config, &var.expr,
            context.tree, context.garden);
        result.push(value);
    }

    for (idx, value) in result.iter().enumerate() {
        multi_var.values[idx].value = Some(value.to_string());
    }

    return result;
}


/// Evaluate environments
pub fn environment(
    config: &mut model::Configuration,
    context: &model::TreeContext,
) -> Vec<(String, String)> {

    let mut result = Vec::new();

    let mut vars = Vec::new();
    for var in &config.trees[context.tree].environment {
        vars.push(var.clone());
    }
    if let Some(garden) = context.garden {
        for var in &config.gardens[garden].environment {
            vars.push(var.clone());
        }
    }

    let mut var_values = Vec::new();
    for var in vars.iter_mut() {
        var_values.push((
            var.name.to_string(),
            multi_variable(config, var, context)
        ));
    }

    // Loop over each value and evaluate the environment command.
    // For "FOO=" values, record a simple (key, value), and update
    // the values dict.  For "FOO" append values, check if it exists
    // in values; if not, check the environment and bootstrap values.
    // If still nothing, initialize it with the value and update the
    // values hashmap.
    let mut values: HashMap<String, String> = HashMap::new();

    for (var_name, env_values) in &var_values {
        let mut name = var_name.to_string();
        let mut is_assign = false;
        let mut is_append = false;

        if name.ends_with("=") {
            is_assign = true;
        }

        if name.ends_with("+") {
            is_append = true;
        }

        if is_assign || is_append {
            let len = name.len();
            name.remove(len - 1);
        }

        for value in env_values {
            let mut current = String::new();
            let mut exists = false;
            if let Some(map_value) = values.get(&name) {
                // Use the existing value
                current = map_value.to_string();
                exists = true;
            }
            if !exists {
                // Not found, try to get the current value from the environment
                let mut has_env = false;
                if let Ok(env_value) = std::env::var(&name) {
                    current = env_value.to_string();
                    has_env = true;
                }

                if has_env && !is_assign {
                    values.insert(name.to_string(), current.to_string());
                } else {
                    // Either no environment value or an assignment will
                    // create the value if it's never been seen.
                    values.insert(name.to_string(), value.to_string());
                    result.push((name.to_string(), value.to_string()));
                    continue;
                }
            }

            // If it's an assignment, replace the value.
            if is_assign {
                values.insert(name.to_string(), value.to_string());
                result.push((name.to_string(), value.to_string()));
                continue;
            }

            // Append/prepend the value.
            let mut path_values: Vec<String> = Vec::new();
            if !is_append {
                path_values.push(value.to_string());
            }
            for path in current.split(':') {
                path_values.push(path.to_string());
            }
            if is_append {
                path_values.push(value.to_string());
            }

            let path_value = path_values.join(":");
            values.insert(name.to_string(), path_value.to_string());
            result.push((name.to_string(), path_value.to_string()));
        }
    }

    return result;
}


/// Evaluate commands
pub fn command(
    config: &mut model::Configuration,
    context: &model::TreeContext,
    name: &str,
) -> Vec<Vec<String>>
{
    let mut vars = Vec::new();
    let pattern = glob::Pattern::new(name).unwrap();

    // Global commands
    for var in &config.commands {
        if pattern.matches(&var.name) {
            vars.push(var.clone());
        }
    }

    // Tree commands
    for var in &config.trees[context.tree].commands {
        if pattern.matches(&var.name) {
            vars.push(var.clone());
        }
    }

    // Optional garden command scope
    if let Some(garden) = context.garden {
        for var in &config.gardens[garden].commands {
            if pattern.matches(&var.name) {
                vars.push(var.clone());
            }
        }
    }

    let mut result = Vec::new();
    for var in vars.iter_mut() {
        result.push(
            multi_variable(config, var, context)
        );
    }

    return result;
}
