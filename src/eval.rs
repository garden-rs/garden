use super::cmd;
use super::model;
use super::path;
use super::query;
use super::syntax;

use std::collections::HashMap;

/// Expand variables across all scopes (garden, tree, and global)
fn expand_tree_vars(
    config: &model::Configuration,
    tree_idx: model::TreeIndex,
    garden_idx: Option<model::GardenIndex>,
    name: &str,
) -> Option<String> {
    // Special case $0, $1, .. $N so they can be used in commands.
    if syntax::is_digit(name) {
        return Some(format!("${name}"));
    }

    // Special-case evaluation of ${graft::values}.
    if syntax::is_graft(name) {
        // TODO: make the error messages more precise by including the tree
        // details by looking up the tree in the configuration.
        let (ok, graft_name, _remainder) = syntax::split_graft(name);
        if !ok {
            // Invalid graft returns an empty value.
            return Some(String::new());
        }

        let graft = match config.get_graft(graft_name) {
            Ok(graft) => graft,
            Err(_) => return Some(String::new()),
        };

        // TODO recurse on the remainder and evaluate it using the ConfigId
        // for the graft.
        let _graft_id = match graft.get_id().ok_or(format!("Invalid graft: {name}")) {
            Ok(graft_id) => graft_id,
            Err(_) => return Some(String::new()),
        };
    }

    let mut var_idx: usize = 0;
    let mut found = false;

    // First check for the variable at the garden scope.
    // Garden scope overrides tree and global scope.
    if let Some(garden) = garden_idx {
        for (idx, var) in config.gardens[garden].variables.iter().enumerate() {
            if var.get_name() == name {
                if let Some(var_value) = var.get_value() {
                    return Some(var_value.to_string());
                }
                var_idx = idx;
                found = true;
                break;
            }
        }

        if found {
            let expr = config.gardens[garden].variables[var_idx]
                .get_expr()
                .to_string();
            let result = tree_value(config, &expr, tree_idx, garden_idx);
            config.gardens[garden].variables[var_idx].set_value(result.clone());
            return Some(result);
        }
    }

    // Nothing was found -- check for the variable in tree scope.
    found = false;
    var_idx = 0;

    for (idx, var) in config.trees[tree_idx].variables.iter().enumerate() {
        if var.get_name() == name {
            if let Some(var_value) = var.get_value() {
                return Some(var_value.to_string());
            }
            found = true;
            var_idx = idx;
            break;
        }
    }

    if found {
        let expr = config.trees[tree_idx].variables[var_idx]
            .get_expr()
            .to_string();
        let result = tree_value(config, &expr, tree_idx, garden_idx);
        config.trees[tree_idx].variables[var_idx].set_value(result.to_string());
        return Some(result);
    }

    // Nothing was found.  Check for the variable in global/config scope.
    found = false;
    var_idx = 0;

    for (idx, var) in config.variables.iter().enumerate() {
        if var.get_name() == name {
            // Return the value immediately if it's already been evaluated.
            if let Some(var_value) = var.get_value() {
                return Some(var_value.to_string());
            }
            found = true;
            var_idx = idx;
            break;
        }
    }

    if found {
        let expr = config.variables[var_idx].get_expr().to_string();
        let result = tree_value(config, &expr, tree_idx, garden_idx);
        config.variables[var_idx].set_value(result.clone());
        return Some(result);
    }

    // If nothing was found then check for environment variables.
    if let Ok(env_value) = std::env::var(name) {
        return Some(env_value);
    }

    // Nothing was found -> empty value
    Some(String::new())
}

/// Expand variables using a tree context.
fn _expand_tree_context_vars(
    _app: &model::ApplicationContext,
    _tree_context: &model::TreeContext,
    _name: &str,
) -> Result<Option<String>, String> {
    Ok(None)
}

/// Expand variables at global scope only
fn expand_vars(config: &model::Configuration, name: &str) -> Option<String> {
    // Special case $0, $1, .. $N so they can be used in commands.
    if syntax::is_digit(name) {
        return Some(format!("${name}"));
    }

    let mut var_idx: usize = 0;
    let mut found = false;

    for (idx, var) in config.variables.iter().enumerate() {
        if var.get_name() == name {
            if let Some(var_value) = var.get_value() {
                return Some(var_value.to_string());
            }
            var_idx = idx;
            found = true;
            break;
        }
    }

    if found {
        let expr = config.variables[var_idx].get_expr().to_string();
        let result = value(config, &expr);
        config.variables[var_idx].set_value(result.clone());

        return Some(result);
    }

    // If nothing was found then check for environment variables.
    if let Ok(env_value) = std::env::var(name) {
        return Some(env_value);
    }

    // Nothing was found -> empty value
    Some(String::new())
}

/// Resolve ~ to the current user's home directory
fn home_dir() -> Option<std::path::PathBuf> {
    // Honor $HOME when set in the environment.
    if let Ok(home) = std::env::var("HOME") {
        return Some(std::path::PathBuf::from(home));
    }
    dirs::home_dir()
}

/// Resolve an expression in a garden/tree/global scope
pub fn tree_value(
    config: &model::Configuration,
    expr: &str,
    tree_idx: model::TreeIndex,
    garden_idx: Option<model::GardenIndex>,
) -> String {
    let is_exec = syntax::is_exec(expr);
    let escaped_value;
    let escaped_expr = if is_exec {
        escaped_value = syntax::escape_shell_variables(expr);
        escaped_value.as_str()
    } else {
        expr
    };
    let expanded = shellexpand::full_with_context_no_errors(escaped_expr, home_dir, |x| {
        expand_tree_vars(config, tree_idx, garden_idx, x)
    })
    .to_string();

    // TODO exec_expression_with_path() to use the tree path.
    // NOTE: an environment must not be calculated here otherwise any
    // exec expression will implicitly depend on the entire environment,
    // and potentially many variables (including itself).  Exec expressions
    // always use the default environment.
    if is_exec {
        let pathbuf = config.get_tree_pathbuf(tree_idx);
        exec_expression(&expanded, pathbuf)
    } else {
        expanded
    }
}

/// Resolve an expression in a garden/tree/global scope for execution by a shell.
/// This is used to generate the commands used internally by garden.
pub fn tree_value_for_shell(
    config: &model::Configuration,
    expr: &str,
    tree_idx: model::TreeIndex,
    garden_idx: Option<model::GardenIndex>,
) -> String {
    let is_exec = syntax::is_exec(expr);
    let expanded = shellexpand::full_with_context_no_errors(
        &syntax::escape_shell_variables(expr),
        home_dir,
        |x| expand_tree_vars(config, tree_idx, garden_idx, x),
    )
    .to_string();

    // NOTE: an environment must not be calculated here otherwise any
    // exec expression will implicitly depend on the entire environment,
    // and potentially many variables (including itself).  Exec expressions
    // always use the default environment.
    if is_exec {
        let pathbuf = config.get_tree_pathbuf(tree_idx);
        exec_expression(&expanded, pathbuf)
    } else {
        expanded
    }
}

/// Resolve a variable in configuration/global scope
pub fn value(config: &model::Configuration, expr: &str) -> String {
    let is_exec = syntax::is_exec(expr);
    let escaped_value;
    let escaped_expr = if is_exec {
        escaped_value = syntax::escape_shell_variables(expr);
        escaped_value.as_str()
    } else {
        expr
    };
    let expanded = shellexpand::full_with_context_no_errors(escaped_expr, home_dir, |x| {
        expand_vars(config, x)
    })
    .to_string();

    if is_exec {
        exec_expression(&expanded, None)
    } else {
        expanded
    }
}

/// Evaluate `$ <command>` command strings, AKA "exec expressions".
/// The result of the expression is the stdout output from the command.
pub fn exec_expression(string: &str, pathbuf: Option<std::path::PathBuf>) -> String {
    let cmd = syntax::trim_exec(string);
    let mut proc = subprocess::Exec::shell(cmd).stdout(subprocess::Redirection::Pipe);
    // Run the exec expression inside the tree's directory when specified.
    if let Some(pathbuf) = pathbuf {
        let current_dir = path::current_dir_string();
        proc = proc.cwd(pathbuf.clone());
        // Set $PWD to ensure that commands that are sensitive to it see the right value.
        proc = proc.env("PWD", pathbuf.to_str().unwrap_or(&current_dir));
    }
    let capture = proc.capture();
    if let Ok(x) = capture {
        return cmd::trim_stdout(&x);
    }
    // An error occurred running the command -- empty output by design
    String::new()
}

/// Evaluate a variable in the given context
pub fn multi_variable(
    config: &model::Configuration,
    multi_var: &mut model::MultiVariable,
    context: &model::TreeContext,
) -> Vec<String> {
    let mut result = Vec::new();

    for var in multi_var.iter() {
        if let Some(value) = var.get_value() {
            result.push(value.to_string());
            continue;
        }

        let value = tree_value(config, var.get_expr(), context.tree, context.garden);
        result.push(value.clone());

        var.set_value(value);
    }

    result
}

/// Evaluate a variable in the given context for execution in a shell
pub fn multi_variable_for_shell(
    config: &model::Configuration,
    multi_var: &mut model::MultiVariable,
    context: &model::TreeContext,
) -> Vec<String> {
    let mut result = Vec::new();

    for var in multi_var.iter() {
        if let Some(value) = var.get_value() {
            result.push(value.to_string());
            continue;
        }

        let value = tree_value_for_shell(config, var.get_expr(), context.tree, context.garden);
        result.push(value.clone());

        var.set_value(value);
    }

    result
}

/// Evaluate environments
pub fn environment(
    config: &model::Configuration,
    context: &model::TreeContext,
) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut vars = Vec::new();
    let mut ready = false;

    if let Some(idx) = context.garden {
        // Evaluate garden environments.
        let garden = &config.gardens[idx];
        for ctx in query::trees_from_garden(config, garden) {
            for var in &config.trees[ctx.tree].environment {
                vars.push((ctx.clone(), var.clone()));
            }
        }

        for var in &garden.environment {
            vars.push((context.clone(), var.clone()));
        }
        ready = true;
    } else if let Some(name) = &context.group {
        // Evaluate group environments.
        if let Some(group) = config.groups.get(name) {
            for ctx in query::trees_from_group(config, None, group) {
                for var in &config.trees[ctx.tree].environment {
                    vars.push((ctx.clone(), var.clone()));
                }
            }
            ready = true;
        }
    }

    // Evaluate a single tree environment when not handled above.
    if !ready {
        for var in &config.trees[context.tree].environment {
            vars.push((context.clone(), var.clone()));
        }
    }

    let mut var_values = Vec::new();
    for (ctx, var) in vars.iter_mut() {
        var_values.push((
            tree_value(config, var.get_name(), ctx.tree, ctx.garden),
            multi_variable(config, var, ctx),
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
        let mut name = var_name.clone();
        let mut is_assign = false;
        let mut is_append = false;

        if name.ends_with('=') {
            is_assign = true;
        }

        if name.ends_with('+') {
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
                current = map_value.clone();
                exists = true;
            }
            if !exists {
                // Not found, try to get the current value from the environment
                let mut has_env = false;
                if let Ok(env_value) = std::env::var(&name) {
                    let env_str: String = env_value;
                    // Empty values are treated as not existing to prevent ":foo" or
                    // "foo:" in the final result.
                    if !env_str.is_empty() {
                        current = env_str;
                        has_env = true;
                    }
                }

                if has_env && !is_assign {
                    values.insert(name.clone(), current.clone());
                } else {
                    // Either no environment value or an assignment will
                    // create the value if it's never been seen.
                    values.insert(name.clone(), value.clone());
                    result.push((name.clone(), value.clone()));
                    continue;
                }
            }

            // If it's an assignment, replace the value.
            if is_assign {
                values.insert(name.clone(), value.clone());
                result.push((name.clone(), value.clone()));
                continue;
            }

            // Append/prepend the value.
            let mut path_values: Vec<String> = Vec::new();
            if !is_append {
                path_values.push(value.clone());
            }
            for path in current.split(':') {
                path_values.push(path.into());
            }
            if is_append {
                path_values.push(value.clone());
            }

            let path_value = path_values.join(":");
            values.insert(name.clone(), path_value.clone());
            result.push((name.clone(), path_value));
        }
    }

    result
}

/// Evaluate commands
pub fn command(
    app: &model::ApplicationContext,
    context: &model::TreeContext,
    name: &str,
) -> Vec<Vec<String>> {
    let mut vars = Vec::new();
    let mut result = Vec::new();
    let config = match context.config {
        Some(config_id) => app.get_config(config_id),
        None => app.get_root_config(),
    };

    let pattern = match glob::Pattern::new(name) {
        Ok(value) => value,
        Err(_) => return result,
    };

    // Global commands
    for var in &config.commands {
        if pattern.matches(var.get_name()) {
            vars.push(var.clone());
        }
    }

    // Tree commands
    for var in &config.trees[context.tree].commands {
        if pattern.matches(var.get_name()) {
            vars.push(var.clone());
        }
    }

    // Optional garden command scope
    if let Some(garden) = context.garden {
        for var in &config.gardens[garden].commands {
            if pattern.matches(var.get_name()) {
                vars.push(var.clone());
            }
        }
    }

    for var in vars.iter_mut() {
        result.push(multi_variable_for_shell(config, var, context));
    }

    result
}
