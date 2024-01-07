use anyhow::Result;
use clap::Parser;

use crate::{cmd, display, errors, eval, model, query};

/// Open a shell in a garden environment
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ShellOptions {
    /// Query for trees to build an environment
    query: String,
    /// Tree to chdir into
    tree: Option<String>,
}

pub fn main(app_context: &model::ApplicationContext, options: &ShellOptions) -> Result<()> {
    let config = app_context.get_root_config_mut();
    let contexts = query::resolve_trees(app_context, config, &options.query);
    if contexts.is_empty() {
        return Err(errors::GardenError::EmptyTreeQueryResult(options.query.clone()).into());
    }

    let mut context = contexts[0].clone();

    // If a tree's name in the returned contexts exactly matches the tree
    // query that was used to find it then chdir into that tree.
    // This makes it convenient to have gardens and trees with the same name.
    for ctx in &contexts {
        if ctx.tree == options.query {
            context = ctx.clone();
            break;
        }
    }

    if let Some(tree) = &options.tree {
        let mut found = false;
        if let Some(ctx) = query::tree_from_name(config, tree, None, None) {
            for query_ctx in &contexts {
                if ctx.tree == query_ctx.tree {
                    context = query_ctx.clone();
                    found = true;
                    break;
                }
            }
        } else {
            error!("unable to find '{}': No tree exists with that name", tree);
        }
        if !found {
            error!(
                "'{}' was not found in the tree query '{}'",
                tree, options.query
            );
        }
    }

    // Evaluate garden.shell
    let shell_expr = config.shell.clone();
    let shell = eval::tree_value(
        app_context,
        config,
        &shell_expr,
        &context.tree,
        context.garden.as_ref(),
    );
    if app_context.options.verbose > 1 {
        // Shell quote the list of commands.
        println!(
            "{} {}",
            display::Color::cyan(":"),
            display::Color::green(&shell),
        );
    }

    if let Some(value) = shlex::split(&shell) {
        cmd::exec_in_context(
            app_context,
            config,
            &context,
            /*quiet*/ true,
            /*verbose*/ 0,
            /*dry_run*/ false,
            &value,
        )
        .map_err(|err| err.into())
    } else {
        Err(errors::GardenError::InvalidConfiguration {
            msg: format!("unable to shlex::split '{shell}'"),
        }
        .into())
    }
}
