use anyhow::Result;
use argparse;

use super::super::cmd;
use super::super::errors;
use super::super::model;
use super::super::query;


/// Main entry point for the "garden exec" command
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let options = &mut app.options;
    let config = &mut app.config;

    let mut query = String::new();
    let mut command: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.silence_double_dash(false);
        ap.stop_on_first_argument(true);
        ap.set_description("garden exec - run commands inside gardens");

        ap.refer(&mut query).required()
            .add_argument("query", argparse::Store,
                          "gardens/groups/trees to exec (tree query)");

        ap.refer(&mut command).required()
            .add_argument("command", argparse::List,
                          "command to run over resolved trees");

        options.args.insert(0, "garden exec".to_string());
        cmd::parse_args(ap, options.args.to_vec());
    }

    if options.is_debug("exec") {
        debug!("command: exec");
        debug!("query: {}", query);
        debug!("command: {:?}", command);
    }

    let quiet = options.quiet;
    let verbose = options.verbose;

    exec(config, quiet, verbose, &query, &command)
}


/// Execute a command over every tree in the evaluated tree query.
pub fn exec(
    config: &mut model::Configuration,
    quiet: bool,
    verbose: bool,
    query: &str,
    command: &Vec<String>,
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
    if command.is_empty() {
        return Err(errors::GardenError::Usage.into());
    }

    // Loop over each context, evaluate the tree environment,
    // and run the command.
    for context in &contexts {
        // Skip symlink trees.
        if config.trees[context.tree].is_symlink {
            continue;
        }
        // Run the command in the current context.
        if let Err(errors::GardenError::ExitStatus(status)) =
                cmd::exec_in_context(config, context, quiet, verbose, command) {
            exit_status = status;
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}
