use super::cmd;
use super::model;
use super::path;
use super::query;
use super::syntax;

use std::collections::HashMap;

/// Expand variables across all scopes (garden, tree, and global).
/// - `config`: reference to Configuration
/// - `tree_idx`: index into the tree being evaluated
/// - `garden_name`: optional garden name being evaluated.
/// - `name`: the name of the variable being expanded.
fn expand_tree_vars(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    tree_name: &str,
    garden_name: Option<&model::GardenName>,
    name: &str,
) -> Option<String> {
    // Special case $0, $1, .. $N so they can be used in commands.
    if syntax::is_digit(name) {
        return Some(format!("${name}"));
    }

    // Special-case evaluation of ${graft::values}.
    if syntax::is_graft(name) {
        let (_graft_id, _remainder) = match config.get_graft_id(name) {
            Ok((graft_id, remainder)) => (graft_id, remainder),
            Err(_) => return Some(String::new()),
        };
    }

    // First check for the variable at the garden scope.
    // Garden scope overrides tree and global scope.
    if let Some(garden_name) = garden_name {
        if let Some(var) = config
            .gardens
            .get(garden_name)
            .and_then(|garden| garden.variables.get(name))
        {
            if let Some(var_value) = var.get_value() {
                return Some(var_value.to_string());
            }
            let expr = var.get_expr();
            let result = tree_value(app_context, config, expr, tree_name, Some(garden_name));
            var.set_value(result.clone());
            return Some(result);
        }
    }

    // Nothing was found -- check for the variable in tree scope.
    if let Some(var) = config
        .trees
        .get(tree_name)
        .and_then(|tree| tree.variables.get(name))
    {
        if let Some(var_value) = var.get_value() {
            return Some(var_value.to_string());
        }
        let expr = var.get_expr();
        let result = tree_value(app_context, config, expr, tree_name, garden_name);
        var.set_value(result.to_string());
        return Some(result);
    }

    // Nothing was found.  Check for the variable in global/config scope.
    if let Some(var) = config.variables.get(name) {
        let expr = var.get_expr();
        let result = tree_value(app_context, config, expr, tree_name, garden_name);
        var.set_value(result.clone());
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
fn expand_vars(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    name: &str,
) -> Option<String> {
    // Special case $0, $1, .. $N so they can be used in commands.
    if syntax::is_digit(name) {
        return Some(format!("${name}"));
    }

    if syntax::is_graft(name) {
        let (graft_id, remainder) = match config.get_graft_id(name) {
            Ok((graft_id, remainder)) => (graft_id, remainder),
            Err(_) => return Some(String::new()),
        };
        return expand_graft_vars(app_context, graft_id, remainder);
    }

    // Check for the variable in the current configuration's global scope.
    if let Some(var) = config.variables.get(name) {
        if let Some(var_value) = var.get_value() {
            return Some(var_value.to_string());
        }

        let expr = var.get_expr();
        let result = value(&app_context, config, expr);
        var.set_value(result.clone());

        return Some(result);
    }

    // Walk up the parent hierarchy to resolve variables defined by graft parents.
    if let Some(parent_id) = config.parent_id {
        let parent_config = app_context.get_config(parent_id);
        return expand_vars(app_context, parent_config, name);
    }

    // If nothing was found then check for environment variables.
    if let Ok(env_value) = std::env::var(name) {
        return Some(env_value);
    }

    // Nothing was found -> empty value
    Some(String::new())
}

/// Expand graft variables of the form "graft::name".
fn expand_graft_vars(
    app_context: &model::ApplicationContext,
    graft_id: model::ConfigId,
    name: &str,
) -> Option<String> {
    if syntax::is_graft(name) {
        let (graft_id, remainder) = match app_context.get_config(graft_id).get_graft_id(name) {
            Ok((graft_id, remainder)) => (graft_id, remainder),
            Err(_) => return Some(String::new()),
        };
        return expand_graft_vars(app_context, graft_id, remainder);
    }

    expand_vars(app_context, app_context.get_config(graft_id), name)
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
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    expr: &str,
    tree_name: &str,
    garden_name: Option<&model::GardenName>,
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
        expand_tree_vars(app_context, config, tree_name, garden_name, x)
    })
    .to_string();

    // TODO exec_expression_with_path() to use the tree path.
    // NOTE: an environment must not be calculated here otherwise any
    // exec expression will implicitly depend on the entire environment,
    // and potentially many variables (including itself).  Exec expressions
    // always use the default environment.
    if is_exec {
        let pathbuf = config.get_tree_pathbuf(tree_name);
        exec_expression(&expanded, pathbuf)
    } else {
        expanded
    }
}

/// Resolve an expression in a garden/tree/global scope for execution by a shell.
/// This is used to generate the commands used internally by garden.
pub fn tree_value_for_shell(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    expr: &str,
    tree_name: &model::TreeName,
    garden_name: Option<&model::GardenName>,
) -> String {
    let is_exec = syntax::is_exec(expr);
    let expanded = shellexpand::full_with_context_no_errors(
        &syntax::escape_shell_variables(expr),
        home_dir,
        |x| expand_tree_vars(app_context, config, tree_name, garden_name, x),
    )
    .to_string();

    // NOTE: an environment must not be calculated here otherwise any
    // exec expression will implicitly depend on the entire environment,
    // and potentially many variables (including itself).  Exec expressions
    // always use the default environment.
    if is_exec {
        let pathbuf = config.get_tree_pathbuf(tree_name);
        exec_expression(&expanded, pathbuf)
    } else {
        expanded
    }
}

/// Resolve a variable in configuration/global scope
pub fn value(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    expr: &str,
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
        expand_vars(app_context, config, x)
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
    app_context: &model::ApplicationContext,
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

        let value = tree_value(
            app_context,
            config,
            var.get_expr(),
            &context.tree,
            context.garden.as_ref(),
        );
        result.push(value.clone());

        var.set_value(value);
    }

    result
}

/// Evaluate a variable in the given context for execution in a shell
pub fn variables_for_shell(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    variables: &mut Vec<model::Variable>,
    context: &model::TreeContext,
) -> Vec<String> {
    let mut result = Vec::new();

    for var in variables {
        if let Some(value) = var.get_value() {
            result.push(value.to_string());
            continue;
        }

        let value = tree_value_for_shell(
            app_context,
            config,
            var.get_expr(),
            &context.tree,
            context.garden.as_ref(),
        );
        result.push(value.clone());

        var.set_value(value);
    }

    result
}

/// Evaluate environments
pub fn environment(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    context: &model::TreeContext,
) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut vars = Vec::new();

    // Evaluate environment variables defined at global scope.
    for var in &config.environment {
        vars.push((context.clone(), var.clone()));
    }

    let mut ready = false;
    if let Some(garden_name) = context.garden.as_ref() {
        // Evaluate garden environments.
        if let Some(garden) = &config.gardens.get(garden_name) {
            for ctx in query::trees_from_garden(config, garden) {
                if let Some(tree) = config.trees.get(&ctx.tree) {
                    for var in &tree.environment {
                        vars.push((ctx.clone(), var.clone()));
                    }
                }
            }

            for var in &garden.environment {
                vars.push((context.clone(), var.clone()));
            }
            ready = true;
        }
    } else if let Some(name) = &context.group {
        // Evaluate group environments.
        if let Some(group) = config.groups.get(name) {
            for ctx in query::trees_from_group(config, None, group) {
                if let Some(tree) = config.trees.get(&ctx.tree) {
                    for var in &tree.environment {
                        vars.push((ctx.clone(), var.clone()));
                    }
                }
            }
            ready = true;
        }
    }

    // Evaluate a single tree environment when not handled above.
    if !ready {
        if let Some(tree) = config.trees.get(&context.tree) {
            for var in &tree.environment {
                vars.push((context.clone(), var.clone()));
            }
        }
    }

    let mut var_values = Vec::new();
    for (ctx, var) in vars.iter_mut() {
        var_values.push((
            tree_value(
                app_context,
                config,
                var.get_name(),
                &ctx.tree,
                ctx.garden.as_ref(),
            ),
            multi_variable(app_context, config, var, ctx),
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
    app_context: &model::ApplicationContext,
    context: &model::TreeContext,
    name: &str,
) -> Vec<Vec<String>> {
    let mut vec_variables = Vec::new();
    let mut result = Vec::new();
    let config = match context.config {
        Some(config_id) => app_context.get_config(config_id),
        None => app_context.get_root_config(),
    };

    let pattern = match glob::Pattern::new(name) {
        Ok(value) => value,
        Err(_) => return result,
    };

    // Global commands
    for (var_name, var) in &config.commands {
        if pattern.matches(var_name) {
            vec_variables.push(var.clone());
        }
    }

    // Tree commands
    if let Some(tree) = config.trees.get(&context.tree) {
        for (var_name, var) in &tree.commands {
            if pattern.matches(var_name) {
                vec_variables.push(var.clone());
            }
        }
    }

    // Optional garden command scope
    if let Some(garden_name) = &context.garden {
        if let Some(garden) = &config.gardens.get(garden_name) {
            for (var_name, var) in &garden.commands {
                if pattern.matches(var_name) {
                    vec_variables.push(var.clone());
                }
            }
        }
    }

    for variables in vec_variables.iter_mut() {
        result.push(variables_for_shell(app_context, config, variables, context));
    }

    result
}
