use anyhow::Result;
use clap::{Parser, ValueHint};

use crate::{cmd, display, errors, model, query};

/// Evaluate garden expressions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ExecOptions {
    /// Tree query for the gardens, groups or trees to run the command
    #[arg(value_hint=ValueHint::Other)]
    query: String,
    /// Command to run in the resolved environments
    #[arg(allow_hyphen_values = true, trailing_var_arg = true, required = true, value_hint=ValueHint::CommandWithArguments)]
    command: Vec<String>,
}

/// Main entry point for the "garden exec" command
pub fn main(app_context: &model::ApplicationContext, exec_options: &ExecOptions) -> Result<()> {
    // parse_args(&mut app.options, &mut query, &mut command);
    let quiet = app_context.options.quiet;
    let verbose = app_context.options.verbose;

    if app_context.options.debug_level("exec") > 0 {
        debug!("query: {}", exec_options.query);
        debug!("command: {:?}", exec_options.command);
    }

    let config = app_context.get_root_config_mut();
    exec(
        app_context,
        config,
        quiet,
        verbose,
        &exec_options.query,
        &exec_options.command,
    )
}

/// Execute a command over every tree in the evaluated tree query.
fn exec(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
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
    let contexts = query::resolve_trees(app_context, config, query);
    let mut exit_status: i32 = 0;

    // Loop over each context, evaluate the tree environment,
    // and run the command.
    for context in &contexts {
        let config = match context.config {
            Some(config_id) => app_context.get_config(config_id),
            None => config,
        };
        let tree = match config.trees.get(&context.tree) {
            Some(tree) => tree,
            None => continue,
        };
        // Skip symlink trees.
        if tree.is_symlink {
            continue;
        }
        if verbose > 1 {
            // Shell quote the list of commands.
            let cmd_str = shell_words::join(command);
            println!(
                "{} {}",
                display::Color::cyan(":"),
                display::Color::green(&cmd_str),
            );
        }
        // Run the command in the current context.
        if let Err(errors::GardenError::ExitStatus(status)) =
            cmd::exec_in_context(app_context, config, context, quiet, verbose, command)
        {
            exit_status = status;
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}
