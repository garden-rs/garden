use anyhow::Result;
use clap::Parser;

use crate::{cmd, errors, eval, model, query};

/// Open a shell in a garden environment
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ShellOptions {
    /// Increase verbosity level (default: 0)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Query for trees to build an environment
    #[arg(default_value = ".")]
    query: String,
    /// Tree to chdir into
    tree: Option<String>,
}

pub fn main(app_context: &model::ApplicationContext, options: &ShellOptions) -> Result<()> {
    let config = app_context.get_root_config_mut();
    let contexts = query::resolve_trees(app_context, config, None, &options.query);
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
    let graft_config = context.config.map(|id| app_context.get_config(id));
    let shell_expr = if config.interactive_shell.is_empty() {
        &config.shell
    } else {
        &config.interactive_shell
    };
    let shell = eval::tree_value(
        app_context,
        config,
        graft_config,
        shell_expr,
        &context.tree,
        context.garden.as_ref(),
    );

    let verbose = app_context.options.verbose + options.verbose;
    let quiet = verbose == 0;
    if let Some(value) = shlex::split(&shell) {
        cmd::exec_in_context(
            app_context,
            config,
            &context,
            quiet,
            verbose,
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
