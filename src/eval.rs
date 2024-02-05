use std::collections::HashMap;

use crate::{cmd, constants, model, path, query, syntax};

/// Expand variables across all scopes (garden, tree, and global).
/// - `app_context`: reference to the top-level ApplicationContext.
/// - `config`: reference to Configuration to use for evaluation
/// - `tree_name`: name of the tree being evaluated.
/// - `garden_name`: optional garden name being evaluated.
/// - `name`: the name of the variable being expanded.
fn expand_tree_vars(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    tree_name: &str,
    garden_name: Option<&model::GardenName>,
    name: &str,
) -> Option<String> {
    // Special case $0, $1, .. $N so they can be used in commands.
    if syntax::is_digit(name) {
        return Some(format!("${name}"));
    }
    // Check for the variable in override scope defined by "garden -D name=value".
    if let Some(var) = config.override_variables.get(name) {
        return Some(tree_variable(
            app_context,
            config,
            graft_config,
            tree_name,
            garden_name,
            var,
        ));
    }

    // Special-case evaluation of ${graft::values}.
    if syntax::is_graft(name) {
        // First, try the current config.
        if let Ok((graft_id, remainder)) = config.get_graft_id(name) {
            return expand_tree_vars(
                app_context,
                config,
                Some(app_context.get_config(graft_id)),
                tree_name,
                garden_name,
                remainder,
            );
        }

        // If nothing was found then try the parent config.
        if let Some(parent_id) = config.parent_id {
            let parent_config = app_context.get_config(parent_id);
            if let Ok((graft_id, remainder)) = parent_config.get_graft_id(name) {
                return expand_tree_vars(
                    app_context,
                    config,
                    Some(app_context.get_config(graft_id)),
                    tree_name,
                    garden_name,
                    remainder,
                );
            }
        }
        // Lastly, try the root configuraiton.
        if let Ok((graft_id, remainder)) = app_context.get_root_config().get_graft_id(name) {
            return expand_tree_vars(
                app_context,
                config,
                Some(app_context.get_config(graft_id)),
                tree_name,
                garden_name,
                remainder,
            );
        }
    }

    // Check for the variable at the grafted garden scope.
    // Garden scope overrides tree and global scope.
    if let Some(garden_name) = garden_name {
        if let Some(var) = graft_config
            .and_then(|cfg| cfg.gardens.get(garden_name))
            .and_then(|garden| garden.variables.get(name))
        {
            return Some(tree_variable(
                app_context,
                config,
                graft_config,
                tree_name,
                Some(garden_name),
                var,
            ));
        }

        // Check for the variable at the root garden scope.
        if let Some(var) = config
            .gardens
            .get(garden_name)
            .and_then(|garden| garden.variables.get(name))
        {
            return Some(tree_variable(
                app_context,
                config,
                graft_config,
                tree_name,
                Some(garden_name),
                var,
            ));
        }
    }

    // Nothing was found -- check for the variable in grafted scopes.
    if let Some(graft_cfg) = graft_config {
        if let Some(var) = graft_cfg
            .trees
            .get(tree_name)
            .and_then(|tree| tree.variables.get(name))
        {
            return Some(tree_variable(
                app_context,
                config,
                graft_config,
                tree_name,
                garden_name,
                var,
            ));
        }
        // Nothing was found. Check for the variable in global/config scope.
        if let Some(var) = graft_cfg.variables.get(name) {
            return Some(tree_variable(
                app_context,
                config,
                graft_config,
                tree_name,
                garden_name,
                var,
            ));
        }
    }

    // Nothing was found -- check for the variable in tree scope.
    if let Some(var) = config
        .trees
        .get(tree_name)
        .and_then(|tree| tree.variables.get(name))
    {
        return Some(tree_variable(
            app_context,
            config,
            graft_config,
            tree_name,
            garden_name,
            var,
        ));
    }
    if name == constants::TREE_NAME {
        return Some(tree_name.to_string());
    }

    // Nothing was found. Check for the variable in global/config scope.
    if let Some(var) = config.variables.get(name) {
        return Some(tree_variable(
            app_context,
            config,
            graft_config,
            tree_name,
            garden_name,
            var,
        ));
    }

    // Nothing was found. Check for garden environment variables.
    let context = model::TreeContext::new(
        tree_name,
        graft_config.and_then(|cfg| cfg.get_id()),
        garden_name.cloned(),
        None,
    );
    if let Some(environ) = environment_value(app_context, config, graft_config, &context, name) {
        return Some(environ);
    }

    // If nothing was found then check for OS environment variables.
    if let Ok(env_value) = std::env::var(name) {
        return Some(env_value);
    }

    // Nothing was found -> empty value
    Some(String::new())
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
        return Some(variable(app_context, config, var));
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
fn home_dir() -> Option<String> {
    // Honor $HOME when set in the environment.
    if let Ok(home) = std::env::var(constants::ENV_HOME) {
        return Some(home);
    }
    dirs::home_dir().map(|x| x.to_string_lossy().to_string())
}

/// Resolve an expression in a garden/tree/global scope
pub fn tree_value(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
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
        expand_tree_vars(app_context, config, graft_config, tree_name, garden_name, x)
    })
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

/// Resolve an expression in a garden/tree/global scope for execution by a shell.
/// This is used to generate the commands used internally by garden.
fn tree_value_for_shell(
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
        |x| expand_tree_vars(app_context, config, None, tree_name, garden_name, x),
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
fn exec_expression(string: &str, pathbuf: Option<std::path::PathBuf>) -> String {
    let cmd = syntax::trim_exec(string);
    let mut proc = subprocess::Exec::shell(cmd);
    // Run the exec expression inside the tree's directory when specified.
    if let Some(pathbuf) = pathbuf {
        let current_dir = path::current_dir_string();
        proc = proc.cwd(pathbuf.clone());
        // Set $PWD to ensure that commands that are sensitive to it see the right value.
        proc = proc.env(constants::ENV_PWD, pathbuf.to_str().unwrap_or(&current_dir));
    }

    cmd::stdout_to_string(proc).unwrap_or_default()
}

/// Evaluate a variable in the given context
pub fn multi_variable(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    multi_var: &mut model::MultiVariable,
    context: &model::TreeContext,
) -> Vec<String> {
    let mut result = Vec::new();
    for var in multi_var.iter() {
        let value = tree_variable(
            app_context,
            config,
            graft_config,
            &context.tree,
            context.garden.as_ref(),
            var,
        );
        result.push(value.to_string());
    }

    result
}

/// Evaluate a variable in the given context for execution in a shell
pub(crate) fn variables_for_shell(
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
        vars.push((context.clone(), var));
    }

    let mut ready = false;
    if let Some(garden_name) = context.garden.as_ref() {
        // Evaluate garden environments.
        if let Some(garden) = &config.gardens.get(garden_name) {
            for ctx in query::trees_from_garden(app_context, config, None, garden) {
                if let Some(tree) = ctx
                    .config
                    .and_then(|id| app_context.get_config(id).trees.get(&ctx.tree))
                {
                    for var in &tree.environment {
                        vars.push((ctx.clone(), var));
                    }
                } else if let Some(tree) = config.trees.get(&ctx.tree) {
                    for var in &tree.environment {
                        vars.push((ctx.clone(), var));
                    }
                }
            }

            for var in &garden.environment {
                vars.push((context.clone(), &var));
            }
            ready = true;
        }
    } else if let Some(name) = &context.group {
        // Evaluate group environments.
        if let Some(group) = config.groups.get(name) {
            for ctx in query::trees_from_group(app_context, config, None, None, group) {
                if let Some(tree) = ctx
                    .config
                    .and_then(|id| app_context.get_config(id).trees.get(&ctx.tree))
                {
                    for var in &tree.environment {
                        vars.push((ctx.clone(), var));
                    }
                } else if let Some(tree) = config.trees.get(&ctx.tree) {
                    for var in &tree.environment {
                        vars.push((ctx.clone(), var));
                    }
                }
            }
            ready = true;
        }
    }

    // Evaluate a single tree environment when not handled above.
    let single_tree;
    if !ready {
        if let Some(tree) = config.trees.get(&context.tree) {
            single_tree = tree;
            for var in &single_tree.environment {
                vars.push((context.clone(), var));
            }
        }
    }

    let mut var_values = Vec::new();
    for (ctx, var) in vars.iter_mut() {
        let mut cloned_var = var.clone();
        let graft_config = ctx.config.map(|id| app_context.get_config(id));
        let values = multi_variable(app_context, config, graft_config, &mut cloned_var, ctx);
        var_values.push((
            tree_value(
                app_context,
                config,
                graft_config,
                var.get_name(),
                ctx.tree.as_str(),
                ctx.garden.as_ref(),
            ),
            values,
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

        if syntax::is_replace_op(&name) {
            is_assign = true;
        }

        if syntax::is_append_op(&name) {
            is_append = true;
        }

        if is_assign || is_append {
            syntax::trim_op_inplace(&mut name);
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

/// Return a vector of references to variables that reference the specified names.
fn environment_value_vars<'a>(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    context: &model::TreeContext,
    names: &[String],
    variables: &'a Vec<model::MultiVariable>,
) -> Vec<(model::TreeContext, String, &'a model::MultiVariable)> {
    let mut vars = Vec::with_capacity(variables.len());
    for var in variables {
        if names.contains(var.get_name()) {
            vars.push((context.clone(), var.get_name().to_string(), var));
            continue;
        }
        if syntax::is_eval_candidate(var.get_name()) {
            let name_value = tree_value(
                app_context,
                config,
                graft_config.or(context.config.map(|cfg_id| app_context.get_config(cfg_id))),
                var.get_name(),
                &context.tree,
                context.garden.as_ref(),
            );
            if names.contains(&name_value) {
                vars.push((context.clone(), name_value, var));
            }
        }
    }

    vars
}

/// Evaluate a single environment variable value.
fn environment_value(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    context: &model::TreeContext,
    name: &str,
) -> Option<String> {
    let mut vars = Vec::new();
    let name_prepend = name.to_string();
    let name_append = format!("{name}+");
    let name_replace = format!("{name}=");
    let names = vec![name_prepend, name_append, name_replace];

    // Evaluate environment variables defined at global scope.
    vars.append(&mut environment_value_vars(
        app_context,
        config,
        graft_config,
        context,
        &names,
        &config.environment,
    ));

    if let Some(graft_cfg) = graft_config {
        vars.append(&mut environment_value_vars(
            app_context,
            config,
            graft_config,
            context,
            &names,
            &graft_cfg.environment,
        ));
    }

    // Evaluate garden environments.
    let mut ready = false;
    if let Some(garden_name) = context.garden.as_ref() {
        let mut garden_ref: Option<&model::Garden> = None;
        if let Some(garden) = graft_config.and_then(|cfg| cfg.gardens.get(garden_name)) {
            garden_ref = Some(garden);
        } else if let Some(garden) = config.gardens.get(garden_name) {
            garden_ref = Some(garden);
        }
        if let Some(garden) = garden_ref {
            for ctx in query::trees_from_garden(app_context, config, graft_config, garden) {
                let garden_graft_config =
                    ctx.config.map(|graft_id| app_context.get_config(graft_id));
                if let Some(tree) = garden_graft_config.and_then(|cfg| cfg.trees.get(&ctx.tree)) {
                    vars.append(&mut environment_value_vars(
                        app_context,
                        config,
                        garden_graft_config,
                        &ctx,
                        &names,
                        &tree.environment,
                    ));
                } else if let Some(tree) = config.trees.get(&ctx.tree) {
                    vars.append(&mut environment_value_vars(
                        app_context,
                        config,
                        graft_config,
                        &ctx,
                        &names,
                        &tree.environment,
                    ));
                }
            }
            // Garden environment variables prepend over tree environment variables.
            vars.append(&mut environment_value_vars(
                app_context,
                config,
                graft_config,
                context,
                &names,
                &garden.environment,
            ));
            ready = true;
        }
    } else if let Some(group_name) = context.group.as_ref() {
        // Evaluate group environments.
        if let Some(group) = config.groups.get(group_name) {
            for ctx in query::trees_from_group(app_context, config, graft_config, None, group) {
                let group_graft_config =
                    ctx.config.map(|graft_id| app_context.get_config(graft_id));
                if let Some(graft_cfg) = group_graft_config {
                    if let Some(tree) = graft_cfg.trees.get(&ctx.tree) {
                        vars.append(&mut environment_value_vars(
                            app_context,
                            config,
                            group_graft_config,
                            &ctx,
                            &names,
                            &tree.environment,
                        ));
                        ready = true;
                    }
                } else if let Some(tree) = config.trees.get(&ctx.tree) {
                    vars.append(&mut environment_value_vars(
                        app_context,
                        config,
                        group_graft_config,
                        &ctx,
                        &names,
                        &tree.environment,
                    ));
                    ready = true;
                }
            }
        }
    }

    // Evaluate a single tree environment when not handled above.
    let single_tree;
    if !ready {
        if let Some(graft_cfg) = graft_config {
            if let Some(tree) = graft_cfg.trees.get(&context.tree) {
                single_tree = tree;
                vars.append(&mut environment_value_vars(
                    app_context,
                    config,
                    graft_config,
                    context,
                    &names,
                    &single_tree.environment,
                ));
            }
        } else if let Some(tree) = config.trees.get(&context.tree) {
            single_tree = tree;
            vars.append(&mut environment_value_vars(
                app_context,
                config,
                graft_config,
                context,
                &names,
                &single_tree.environment,
            ));
        }
    }

    let mut var_values = Vec::new();
    for (ctx, name_value, var) in vars.iter_mut() {
        let mut cloned_var = var.clone();
        let values = multi_variable(
            app_context,
            config,
            graft_config.or(ctx.config.map(|id| app_context.get_config(id))),
            &mut cloned_var,
            ctx,
        );
        var_values.push((name_value, values));
    }

    // Loop over each value and evaluate the environment command.
    // For "FOO=" values, record a simple (key, value), and update
    // the values dict.  For "FOO" append values, check if it exists
    // in values; if not, check the environment and bootstrap values.
    // If still nothing, initialize it with the value and update the
    // values hashmap.
    let mut final_value: Option<String> = None;

    for (var_name, env_values) in var_values {
        let mut real_name = var_name.clone();
        let mut is_assign = false;
        let mut is_append = false;

        if syntax::is_replace_op(var_name) {
            is_assign = true;
        }

        if syntax::is_append_op(var_name) {
            is_append = true;
        }

        if is_assign || is_append {
            syntax::trim_op_inplace(&mut real_name);
        }

        for value in env_values {
            let mut current = String::new();
            let exists = if let Some(final_string_value) = final_value {
                // Use the existing value
                current = final_string_value.clone();
                true
            } else {
                false
            };
            if !exists {
                // Not found, try to get the current value from the environment
                let mut has_env = false;
                if let Ok(env_value) = std::env::var(&real_name) {
                    let env_str: String = env_value;
                    // Empty values are treated as not existing to prevent ":foo" or
                    // "foo:" in the final result.
                    if !env_str.is_empty() {
                        current = env_str;
                        has_env = true;
                    }
                }

                #[allow(unused_assignments)]
                if has_env && !is_assign {
                    final_value = Some(current.clone());
                } else {
                    // Either no environment value or an assignment will
                    // create the value if it's never been seen.
                    final_value = Some(value.clone());
                    continue;
                }
            }

            // If it's an assignment, replace the value.
            if is_assign {
                final_value = Some(value.clone());
                continue;
            }

            // Append/prepend the value.
            let mut path_values: Vec<String> = Vec::new();
            if !is_append {
                path_values.push(value.clone());
            }
            for path in current.split(':') {
                if !path.is_empty() {
                    path_values.push(path.to_string());
                }
            }
            if is_append {
                path_values.push(value.clone());
            }

            let path_value = path_values.join(":");
            final_value = Some(path_value.clone());
        }
    }

    final_value
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

/// Evaluate a variable with a tree context if it has not already been evaluated.
pub(crate) fn tree_variable(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    tree_name: &str,
    garden_name: Option<&model::GardenName>,
    var: &model::Variable,
) -> String {
    if let Some(var_value) = var.get_value() {
        return var_value.to_string();
    }
    let expr = var.get_expr();
    let result = tree_value(
        app_context,
        config,
        graft_config,
        expr,
        tree_name,
        garden_name,
    );
    var.set_value(result.to_string());

    result
}

/// Evaluate a variable if it has not already been evaluated.
pub(crate) fn variable(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    var: &model::Variable,
) -> String {
    if let Some(var_value) = var.get_value() {
        return var_value.to_string();
    }
    let expr = var.get_expr();
    let result = value(app_context, config, expr);
    var.set_value(result.to_string());

    result
}
