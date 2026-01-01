use std::sync::atomic;

use anyhow::Result;
use clap::{Parser, ValueHint};
use rayon::prelude::*;

use crate::cli::GardenOptions;
use crate::{cmd, constants, errors, model, query};

/// Evaluate garden expressions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ExecOptions {
    /// Filter trees by name post-query using a glob pattern
    #[arg(long, short, default_value = "*")]
    trees: String,
    /// Perform a trial run without executing any commands
    #[arg(long, short = 'N', short_alias = 'n')]
    dry_run: bool,
    /// Run commands in parallel using the specified number of jobs.
    #[arg(long = "jobs", short = 'j', value_name = "JOBS")]
    num_jobs: Option<usize>,
    /// Be quiet
    #[arg(short, long)]
    quiet: bool,
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
pub fn main(app_context: &model::ApplicationContext, exec_options: &mut ExecOptions) -> Result<()> {
    exec_options.verbose += app_context.options.verbose;
    if app_context.options.debug_level(constants::DEBUG_LEVEL_EXEC) > 0 {
        debug!("query: {}", exec_options.query);
        debug!("command: {:?}", exec_options.command);
    }
    exec_options.quiet |= app_context.options.quiet;
    exec(app_context, exec_options)
}

/// Execute a command over every tree in the evaluated tree query.
fn exec(app_context: &model::ApplicationContext, exec_options: &ExecOptions) -> Result<()> {
    let quiet = exec_options.quiet;
    let verbose = exec_options.verbose;
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
    cmd::initialize_threads_option(exec_options.num_jobs)?;

    // Resolve the tree query into a vector of tree contexts.
    let config = app_context.get_root_config_mut();
    let contexts = query::resolve_trees(app_context, config, None, query);
    let pattern = glob::Pattern::new(tree_pattern).unwrap_or_default();
    let exit_status = atomic::AtomicI32::new(errors::EX_OK);

    // Loop over each context, evaluate the tree environment,
    // and run the command.
    if exec_options.num_jobs.is_some() {
        contexts.par_iter().for_each(|context| {
            let app_context_clone = app_context.clone();
            let app_context = &app_context_clone;
            if !model::is_valid_context(app_context, &pattern, context) {
                return;
            }
            // Run the command in the current context.
            if let Err(errors::GardenError::ExitStatus(status)) = cmd::exec_in_context(
                app_context,
                app_context.get_root_config(),
                context,
                quiet,
                verbose,
                dry_run,
                command,
            ) {
                exit_status.store(status, atomic::Ordering::Release);
            }
        });
    } else {
        for context in &contexts {
            if !model::is_valid_context(app_context, &pattern, context) {
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
                exit_status.store(status, atomic::Ordering::Release);
            }
        }
    }

    // Return the last non-zero exit status.
    errors::exit_status_into_result(exit_status.load(atomic::Ordering::Acquire))
}
