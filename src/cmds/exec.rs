use anyhow::Result;
use clap::Parser;

use super::super::cmd;
use super::super::errors;
use super::super::model;
use super::super::query;

/// Evaluate garden expressions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ExecOptions {
    /// Tree query for the gardens, groups or trees to run the command
    query: String,
    /// Command to run in the resolved environments
    #[arg(allow_hyphen_values = true, required = true)]
    command: Vec<String>,
}

/// Main entry point for the "garden exec" command
pub fn main(app: &mut model::ApplicationContext, exec_options: &ExecOptions) -> Result<()> {
    // parse_args(&mut app.options, &mut query, &mut command);
    let quiet = app.options.quiet;
    let verbose = app.options.verbose;

    if app.options.debug_level("exec") > 0 {
        debug!("query: {}", exec_options.query);
        debug!("command: {:?}", exec_options.command);
    }

    let config = app.get_root_config_mut();
    exec(
        config,
        quiet,
        verbose,
        &exec_options.query,
        &exec_options.command,
    )
}

/// Execute a command over every tree in the evaluated tree query.
pub fn exec(
    config: &mut model::Configuration,
    quiet: bool,
    verbose: u8,
    query: &str,
    command: &[String],
) -> Result<()> {
    // Strategy: resolve the trees down to a set of tree indexes paired with an
    // an optional garden context.
    //
    // If the names resolve to gardens, each garden is processed independently.
    // Trees that exist in multiple matching gardens will be processed multiple
    // times.
    //
    // If the names resolve to trees, each tree is processed independently
    // with no garden context.

    // Resolve the tree query into a vector of tree contexts.
    let contexts = query::resolve_trees(config, query);
    let mut exit_status: i32 = 0;

    // Loop over each context, evaluate the tree environment,
    // and run the command.
    for context in &contexts {
        // Skip symlink trees.
        if config.trees[context.tree].is_symlink {
            continue;
        }
        // Run the command in the current context.
        if let Err(errors::GardenError::ExitStatus(status)) =
            cmd::exec_in_context(config, context, quiet, verbose, command)
        {
            exit_status = status;
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}
