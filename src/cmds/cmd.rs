use anyhow::Result;
use subprocess;

use super::super::cmd;
use super::super::errors;
use super::super::eval;
use super::super::model;
use super::super::query;


/// garden cmd <query> <command>...
pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let mut query = String::new();
    let mut commands = Vec::new();
    let mut arguments = Vec::new();
    parse_args(&mut app.options, &mut query, &mut commands, &mut arguments);

    let quiet = app.options.quiet;
    let verbose = app.options.verbose;
    let keep_going = app.options.keep_going;
    let exit_status = cmd(
        app,
        quiet,
        verbose,
        keep_going,
        &query,
        &commands,
        &arguments,
    )?;
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}


/// Parse "cmd" arguments.
fn parse_args(
    options: &mut model::CommandOptions,
    query: &mut String,
    commands: &mut Vec<String>,
    arguments: &mut Vec<String>,
) {
    let mut commands_and_args: Vec<String> = Vec::new();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.silence_double_dash(false);
        ap.set_description("garden cmd - run custom commands over gardens");

        ap.refer(&mut options.keep_going).add_option(
            &["-k", "--keep-going"],
            argparse::StoreTrue,
            "continue to the next tree when errors occur",
        );

        ap.refer(query).required().add_argument(
            "query",
            argparse::Store,
            "gardens/groups/trees to exec (tree query)",
        );

        ap.refer(&mut commands_and_args).required().add_argument(
            "commands",
            argparse::List,
            "commands to run over resolved trees",
        );

        options.args.insert(0, "garden cmd".into());
        cmd::parse_args(ap, options.args.to_vec());
    }

    if options.is_debug("cmd") {
        debug!("subcommand: cmd");
        debug!("query: {}", query);
        debug!("commands_and_args: {:?}", commands_and_args);
    }

    // Queries and arguments are separated by a double-dash "--" marker.
    cmd::split_on_dash(&commands_and_args, commands, arguments);

    if options.is_debug("cmd") {
        debug!("commands: {:?}", commands);
        debug!("arguments: {:?}", arguments);
    }
}


/// garden <command> <query>...
pub fn custom(app: &mut model::ApplicationContext, command: &str) -> Result<()> {
    let mut queries = Vec::new();
    let mut arguments = Vec::new();
    parse_args_custom(command, &mut app.options, &mut queries, &mut arguments);

    let quiet = app.options.quiet;
    let verbose = app.options.verbose;
    let keep_going = app.options.keep_going;
    cmds(
        app,
        quiet,
        verbose,
        keep_going,
        command,
        &queries,
        &arguments,
    ).map_err(|err| err.into())
}


/// Parse custom command arguments.
fn parse_args_custom(
    command: &str,
    options: &mut model::CommandOptions,
    queries: &mut Vec<String>,
    arguments: &mut Vec<String>,
) {
    let mut queries_and_arguments: Vec<String> = Vec::new();
    let mut ap = argparse::ArgumentParser::new();
    ap.silence_double_dash(false);
    ap.set_description("garden cmd - run custom commands over gardens");

    ap.refer(&mut options.keep_going).add_option(
        &["-k", "--keep-going"],
        argparse::StoreTrue,
        "continue to the next tree when errors occur",
    );

    ap.refer(&mut queries_and_arguments).add_argument(
        "queries",
        argparse::List,
        "gardens/groups/trees to exec (tree queries)",
    );

    options.args.insert(0, format!("garden {}", command));
    cmd::parse_args(ap, options.args.to_vec());

    if options.is_debug("cmd") {
        debug!("command: {}", command);
        debug!("queries_and_arguments: {:?}", queries_and_arguments);
    }

    // Queries and arguments are separated by a double-dash "--" marker.
    cmd::split_on_dash(&queries_and_arguments, queries, arguments);

    // Default to "." when no queries have been specified.
    if queries.is_empty() {
        queries.push(".".into());
    }

    if options.is_debug("cmd") {
        debug!("queries {:?}", queries);
        debug!("arguments: {:?}", arguments);
    }
}

/// Strategy: resolve the trees down to a set of tree indexes paired with an
/// an optional garden context.
///
/// If the names resolve to gardens, each garden is processed independently.
/// Trees that exist in multiple matching gardens will be processed multiple
/// times.
///
/// If the names resolve to trees, each tree is processed independently
/// with no garden context.

pub fn cmd(
    app: &mut model::ApplicationContext,
    quiet: bool,
    verbose: bool,
    keep_going: bool,
    query: &str,
    commands: &Vec<String>,
    arguments: &Vec<String>,
) -> Result<i32> {
    // Resolve the tree query into a vector of tree contexts.
    let contexts;
    let mut exit_status: i32 = errors::EX_OK;
    // Mutable scope for app.get_root_config_mut()
    {
        let config = app.get_root_config_mut();
        contexts = query::resolve_trees(config, query);
    }

    // Loop over each context, evaluate the tree environment,
    // and run the command.
    for context in &contexts {
        // Skip symlink trees.
        {
            let config = app.get_root_config();
            if config.trees[context.tree].is_symlink {
                continue;
            }
        }
        // Evaluate the tree environment
        let env;
        {
            env = eval::environment(app.get_root_config(), context);
        }

        // Run each command in the tree's context
        let path: String;
        {
            let config = app.get_root_config();
            let tree = &config.trees[context.tree];
            path = tree.path_as_ref()?.to_string();
            // Sparse gardens/missing trees are ok -> skip these entries.
            if !model::print_tree(&tree, verbose, quiet) {
                continue;
            }
        }

        // The "error" flag is set when a non-zero exit status is returned.
        let mut error = false;

        // Get the current executable name
        let current_exe = cmd::current_exe();

        // One invocation runs multiple commands
        for name in commands {
            // One command maps to multiple command sequences.
            // When the scope is tree, only the tree's commands
            // are included.  When the scope includes a gardens,
            // its matching commands are appended to the end.
            error = false;
            let cmd_seq_vec = eval::command(app, context, &name);
            app.get_root_config_mut().reset();

            for cmd_seq in &cmd_seq_vec {
                for cmd_str in cmd_seq {
                    let mut exec = subprocess::Exec::shell(&cmd_str)
                        .arg(&current_exe)
                        .args(arguments)
                        .cwd(&path);
                    // Update the command environment
                    for (k, v) in &env {
                        exec = exec.env(k, v);
                    }
                    let status = cmd::status(exec.join());
                    if status != errors::EX_OK {
                        exit_status = status as i32;
                        error = true;
                        break;
                    }
                }
                if error {
                    break;
                }
            }
            if error {
                break;
            }
        }

        if error && !keep_going {
            break;
        }
    }

    // Return the last non-zero exit status.
    Ok(exit_status)
}


/// Run cmd() over a Vec of tree queries
pub fn cmds(
    app: &mut model::ApplicationContext,
    quiet: bool,
    verbose: bool,
    keep_going: bool,
    command: &str,
    queries: &Vec<String>,
    arguments: &Vec<String>,
) -> Result<()> {
    let mut exit_status: i32 = 0;

    let mut commands: Vec<String> = Vec::new();
    commands.push(command.to_string());

    for query in queries {
        let status = cmd(
            app,
            quiet,
            verbose,
            keep_going,
            &query,
            &commands,
            arguments,
        ).unwrap_or(errors::EX_IOERR);
        if status != 0 {
            exit_status = status;
            if !keep_going {
                break;
            }
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}
