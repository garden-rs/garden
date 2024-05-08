use anyhow::Result;
use clap::{Parser, ValueHint};

use crate::{cmd, constants, errors, model, query};

/// Evaluate garden expressions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ExecOptions {
    /// Filter trees by name post-query using a glob pattern
    #[arg(long, short, default_value = "*")]
    trees: String,
    /// Perform a trial run without executing any commands
    #[arg(long, short = 'n')]
    dry_run: bool,
    /// Increase verbosity level (default: 0)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Tree query for the gardens, groups or trees to run the command
    #[arg(value_hint=ValueHint::Other)]
    query: String,
    /// Command to run in the resolved environments
    #[arg(allow_hyphen_values = true, trailing_var_arg = true, required = true, value_hint=ValueHint::CommandWithArguments)]
    command: Vec<String>,
}

/// Main entry point for the "garden exec" command
pub fn main(app_context: &model::ApplicationContext, exec_options: &ExecOptions) -> Result<()> {
    if app_context.options.debug_level(constants::DEBUG_LEVEL_EXEC) > 0 {
        debug!("query: {}", exec_options.query);
        debug!("command: {:?}", exec_options.command);
    }

    let config = app_context.get_root_config_mut();
    exec(app_context, config, exec_options)
}

/// Execute a command over every tree in the evaluated tree query.
fn exec(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    exec_options: &ExecOptions,
) -> Result<()> {
    let quiet = app_context.options.quiet;
    let verbose = app_context.options.verbose + exec_options.verbose;
    let dry_run = exec_options.dry_run;
    let query = &exec_options.query;
    let tree_pattern = &exec_options.trees;
    let command = &exec_options.command;
    // Strategy: resolve the trees down to a set of tree indexes paired with
    // an optional garden context.
    //
    // If the names resolve to gardens, each garden is processed independently.
    // Trees that exist in multiple matching gardens will be processed multiple
    // times.
    //
    // If the names resolve to trees, each tree is processed independently
    // with no garden context.

    // Resolve the tree query into a vector of tree contexts.
    let contexts = query::resolve_trees(app_context, config, None, query);
    let pattern = glob::Pattern::new(tree_pattern).unwrap_or_default();
    let mut exit_status: i32 = 0;

    // Loop over each context, evaluate the tree environment,
    // and run the command.
    for context in &contexts {
        if !pattern.matches(&context.tree) {
            continue;
        }
        let tree_opt = match context.config {
            Some(graft_id) => app_context.get_config(graft_id).trees.get(&context.tree),
            None => config.trees.get(&context.tree),
        };
        let tree = match tree_opt {
            Some(tree) => tree,
            None => continue,
        };
        // Skip symlink trees.
        if tree.is_symlink {
            continue;
        }
        // Run the command in the current context.
        if let Err(errors::GardenError::ExitStatus(status)) = cmd::exec_in_context(
            app_context,
            config,
            context,
            quiet,
            verbose,
            dry_run,
            command,
        ) {
            exit_status = status;
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}
