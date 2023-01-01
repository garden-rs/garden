use anyhow::Result;

use super::super::cmd;
use super::super::errors;
use super::super::eval;
use super::super::model;
use super::super::query;

/// Main entry point for `garden cmd <query> <command>...`.
pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let (query, params) = parse_args_cmd(&mut app.options);
    let exit_status = cmd(app, &query, &params)?;
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}

/// CmdParams are used to control the execution of run_cmd_vec().
///
/// `garden cmd` and `garden <custom-cmd>` parse command line arguments into CmdParams.
#[derive(Clone, Debug, Default)]
pub struct CmdParams {
    commands: Vec<String>,
    arguments: Vec<String>,
    queries: Vec<String>,
}

impl CmdParams {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Parse "cmd" arguments.
fn parse_args_cmd(options: &mut model::CommandOptions) -> (String, CmdParams) {
    // Display "garden cmd" in the "garden cmd -h" help text.
    options.args.insert(0, "garden cmd".into());

    let mut query = String::new();
    let mut params = CmdParams::new();
    let mut commands_and_args: Vec<String> = Vec::new();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.silence_double_dash(false);
        ap.set_description("garden cmd - Run custom commands over gardens");
        ap.refer(&mut options.breadth_first).add_option(
            &["-b", "--breadth-first"],
            argparse::StoreTrue,
            "Run a command in all trees before running the next command.",
        );
        ap.refer(&mut options.keep_going).add_option(
            &["-k", "--keep-going"],
            argparse::StoreTrue,
            "Continue to the next tree when errors occur.",
        );
        ap.refer(&mut options.exit_on_error).add_option(
            &["-n", "--no-errexit"],
            argparse::StoreFalse,
            "Do not pass \"-e\" to the shell. This prevents the \"errexit\" shell \
            option from being set. By default, the \"-e\" option is passed to the \
            configured shell so that multi-line and multi-statement commands halt \
            execution when the first statement with a non-zero exit code is \
            encountered. \"--no-errexit\" has the effect of making multi-line and \
            multi-statement commands run all statements even when an earlier statement \
            returns a non-zero exit code.",
        );
        ap.refer(&mut query).required().add_argument(
            "query",
            argparse::Store,
            "Gardens/Groups/Trees to exec (tree query).",
        );
        ap.refer(&mut commands_and_args).required().add_argument(
            "commands",
            argparse::List,
            "Commands to run over resolved trees.",
        );
        cmd::parse_args(ap, options.args.to_vec());
    }

    if options.debug_level("cmd") > 0 {
        debug!("subcommand: cmd");
        debug!("query: {}", query);
        debug!("commands_and_args: {:?}", commands_and_args);
    }
    // Queries and arguments are separated by a double-dash "--" marker.
    cmd::split_on_dash(
        &commands_and_args,
        &mut params.commands,
        &mut params.arguments,
    );
    if options.debug_level("cmd") > 0 {
        debug!("commands: {:?}", params.commands);
        debug!("arguments: {:?}", params.arguments);
    }

    (query, params)
}

/// Main entry point for `garden <command> <query>...`.
pub fn custom(app: &mut model::ApplicationContext, command: &str) -> Result<()> {
    let params = parse_args_custom(command, &mut app.options);
    cmds(app, &params)
}

/// Parse custom command arguments.
fn parse_args_custom(command: &str, options: &mut model::CommandOptions) -> CmdParams {
    // Display "garden <command>" in the "garden <commmand> -h" help text.
    options.args.insert(0, format!("garden {}", command));

    // Custom commands run breadth-first. The distinction shouldn't make a difference in practice
    // because "garden <custom-cmd> ..." is only able to run a single command, but we use
    // breadth-first because it retains the original implementation/behavior from before
    // --breadth-first was added to "garden cmd" and made otp-in.
    options.breadth_first = true;

    // Add the custom command name to the list of commands. cmds() operates on a vec of commands.
    let mut params = CmdParams::new();
    params.commands.push(command.to_string());

    let mut queries_and_arguments: Vec<String> = Vec::new();
    let mut ap = argparse::ArgumentParser::new();
    ap.silence_double_dash(false);
    ap.set_description("garden cmd - Run custom commands over gardens");

    ap.refer(&mut options.keep_going).add_option(
        &["-k", "--keep-going"],
        argparse::StoreTrue,
        "Continue to the next tree when errors occur.",
    );
    ap.refer(&mut options.exit_on_error).add_option(
        &["-n", "--no-errexit"],
        argparse::StoreFalse,
        "Do not pass \"-e\" to the shell. This prevents the \"errexit\" shell \
        option from being set. By default, the \"-e\" option is passed to the \
        configured shell so that multi-line and multi-statement commands halt \
        execution when the first statement with a non-zero exit code is \
        encountered. \"--no-errexit\" has the effect of making multi-line and \
        multi-statement commands run all statements even when an earlier statement \
        returns a non-zero exit code.",
    );
    ap.refer(&mut queries_and_arguments).add_argument(
        "queries",
        argparse::List,
        "Gardens/Groups/Trees to exec (tree queries).",
    );

    cmd::parse_args(ap, options.args.to_vec());

    if options.debug_level("cmd") > 0 {
        debug!("command: {}", command);
        debug!("queries_and_arguments: {:?}", queries_and_arguments);
    }

    // Queries and arguments are separated by a double-dash "--" marker.
    cmd::split_on_dash(
        &queries_and_arguments,
        &mut params.queries,
        &mut params.arguments,
    );

    // Default to "." when no queries have been specified.
    if params.queries.is_empty() {
        params.queries.push(".".into());
    }

    if options.debug_level("cmd") > 0 {
        debug!("queries {:?}", params.queries);
        debug!("arguments: {:?}", params.arguments);
    }

    params
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

pub fn cmd(app: &mut model::ApplicationContext, query: &str, params: &CmdParams) -> Result<i32> {
    // Mutable scope for app.get_root_config_mut()
    let config = app.get_root_config_mut();
    // Resolve the tree query into a vector of tree contexts.
    let contexts = query::resolve_trees(config, query);

    if app.options.breadth_first {
        run_cmd_breadth_first(app, &contexts, &params.commands, &params.arguments)
    } else {
        run_cmd_depth_first(app, &contexts, &params.commands, &params.arguments)
    }
}

pub fn run_cmd_breadth_first(
    app: &mut model::ApplicationContext,
    contexts: &[model::TreeContext],
    commands: &[String],
    arguments: &[String],
) -> Result<i32> {
    let mut exit_status: i32 = errors::EX_OK;
    let keep_going = app.options.keep_going;
    let quiet = app.options.quiet;
    let verbose = app.options.verbose;
    let shell = {
        let config = app.get_root_config();
        config.shell.to_string()
    };
    // Loop over each command, evaluate the tree environment,
    // and run the command in each context.
    for name in commands {
        // One invocation runs multiple commands
        for context in contexts {
            // Skip symlink trees.
            let config = app.get_root_config();
            if config.trees[context.tree].is_symlink {
                continue;
            }
            // Evaluate the tree environment
            let env = eval::environment(app.get_root_config(), context);

            // Run each command in the tree's context
            let tree = &config.trees[context.tree];
            let path = tree.path_as_ref()?.to_string();
            // Sparse gardens/missing trees are ok -> skip these entries.
            if !model::print_tree(tree, verbose, quiet) {
                continue;
            }

            // One command maps to multiple command sequences.
            // When the scope is tree, only the tree's commands
            // are included.  When the scope includes a gardens,
            // its matching commands are appended to the end.
            let cmd_seq_vec = eval::command(app, context, name);
            app.get_root_config_mut().reset();

            if let Err(cmd_status) =
                run_cmd_vec(&app.options, &path, &shell, &env, &cmd_seq_vec, arguments)
            {
                exit_status = cmd_status;
                if !keep_going {
                    return Ok(cmd_status);
                }
            }
        }
    }

    // Return the last non-zero exit status.
    Ok(exit_status)
}

pub fn run_cmd_depth_first(
    app: &mut model::ApplicationContext,
    contexts: &[model::TreeContext],
    commands: &[String],
    arguments: &[String],
) -> Result<i32> {
    let mut exit_status: i32 = errors::EX_OK;
    let keep_going = app.options.keep_going;
    let quiet = app.options.quiet;
    let verbose = app.options.verbose;
    let shell = {
        let config = app.get_root_config();
        config.shell.to_string()
    };
    // Loop over each context, evaluate the tree environment and run the command.
    for context in contexts {
        // Skip symlink trees.
        let config = app.get_root_config();
        if config.trees[context.tree].is_symlink {
            continue;
        }
        // Evaluate the tree environment
        let env = eval::environment(app.get_root_config(), context);

        // Run each command in the tree's context
        let tree = &config.trees[context.tree];
        let path = tree.path_as_ref()?.to_string();

        // Sparse gardens/missing trees are ok -> skip these entries.
        if !model::print_tree(tree, verbose, quiet) {
            continue;
        }

        // One invocation runs multiple commands
        for name in commands {
            // One command maps to multiple command sequences.
            // When the scope is tree, only the tree's commands
            // are included.  When the scope includes a gardens,
            // its matching commands are appended to the end.
            let cmd_seq_vec = eval::command(app, context, name);
            app.get_root_config_mut().reset();

            if let Err(cmd_status) =
                run_cmd_vec(&app.options, &path, &shell, &env, &cmd_seq_vec, arguments)
            {
                exit_status = cmd_status;
                if !keep_going {
                    return Ok(cmd_status);
                }
            }
        }
    }

    // Return the last non-zero exit status.
    Ok(exit_status)
}

/// Run a vector of custom commands using the configured shell.
/// Parameters:
/// - path: The current working directory for the command.
/// - shell: The shell that will be used to run the command strings.
/// - env: Environment variables to set.
/// - cmd_seq_vec: Vector of vector of command strings to run.
/// - arguments: Additional command line arguments available in $1, $2, $N.
fn run_cmd_vec(
    options: &model::CommandOptions,
    path: &str,
    shell: &str,
    env: &Vec<(String, String)>,
    cmd_seq_vec: &[Vec<String>],
    arguments: &[String],
) -> Result<(), i32> {
    // Get the current executable name
    let current_exe = cmd::current_exe();
    let mut exit_status = errors::EX_OK;

    for cmd_seq in cmd_seq_vec {
        for cmd_str in cmd_seq {
            if options.verbose > 1 {
                println!(
                    "{} {}",
                    model::Color::cyan(":"),
                    model::Color::green(&cmd_str),
                );
            }
            let mut exec = subprocess::Exec::cmd(shell).cwd(path);
            if options.exit_on_error {
                exec = exec.arg("-e");
            }
            exec = exec
                .arg("-c")
                .arg(cmd_str)
                .arg(current_exe.as_str())
                .args(arguments);
            // Update the command environment
            for (k, v) in env {
                exec = exec.env(k, v);
            }
            let status = cmd::status(exec.join());
            // When a command list is used then the return code from the final command
            // is the one that is returned when --no-errexit is in effect.
            if status != errors::EX_OK {
                exit_status = status;
                if options.exit_on_error {
                    return Err(status);
                }
            } else {
                exit_status = errors::EX_OK;
            }
        }
        if exit_status != errors::EX_OK {
            return Err(exit_status);
        }
    }

    Ok(())
}

/// Run cmd() over a Vec of tree queries
pub fn cmds(app: &mut model::ApplicationContext, params: &CmdParams) -> Result<()> {
    let mut exit_status = errors::EX_OK;
    let keep_going = app.options.keep_going;

    for query in &params.queries {
        let status = cmd(app, query, params).unwrap_or(errors::EX_IOERR);
        if status != errors::EX_OK {
            exit_status = status;
            if !keep_going {
                break;
            }
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}
