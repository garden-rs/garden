use super::super::cli;
use super::super::cmd;
use super::super::errors;
use super::super::eval;
use super::super::model;
use super::super::query;

use anyhow::Result;
use clap;
use clap::{CommandFactory, FromArgMatches, Parser};

/// Run one or more custom commands over a tree query
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct CmdOptions {
    /// Run a command in all trees before running the next command
    #[arg(long, short)]
    breadth_first: bool,
    /// Continue to the next tree when errors occur
    #[arg(long, short)]
    keep_going: bool,
    /// Do not pass "-e" to the shell.
    /// Prevent the "errexit" shell option from being set. By default, the "-e" option
    /// is passed to the configured shell so that multi-line and multi-statement
    /// commands halt execution when the first statement with a non-zero exit code is
    /// encountered. "--no-errexit" has the effect of making multi-line and
    /// multi-statement commands run all statements even when an earlier statement
    /// returns a non-zero exit code.
    #[arg(long, short)]
    no_errexit: bool,
    /// Tree query for the gardens, groups or trees to execute commands within
    query: String,
    /// Custom commands to run over the resolved trees
    // NOTE: value_terminator may not be needed in future versions of clap_complete.
    // https://github.com/clap-rs/clap/pull/4612
    #[arg(required = true, value_terminator = "--")]
    commands: Vec<String>,
    /// Arguments to forward to custom commands
    #[arg(last = true)]
    arguments: Vec<String>,
}

/// Run custom garden commands
#[derive(Parser, Clone, Debug)]
#[command(bin_name = "garden")]
pub struct CustomOptions {
    /// Continue to the next tree when errors occur
    #[arg(long, short)]
    keep_going: bool,
    /// Do not pass "-e" to the shell.
    /// Prevent the "errexit" shell option from being set. By default, the "-e" option
    /// is passed to the configured shell so that multi-line and multi-statement
    /// commands halt execution when the first statement with a non-zero exit code is
    /// encountered. "--no-errexit" has the effect of making multi-line and
    /// multi-statement commands run all statements even when an earlier statement
    /// returns a non-zero exit code.
    #[arg(long, short)]
    no_errexit: bool,
    /// Tree queries for the Gardens/Groups/Trees to execute commands within
    // NOTE: value_terminator may not be needed in future versions of clap_complete.
    // https://github.com/clap-rs/clap/pull/4612
    #[arg(value_terminator = "--")]
    queries: Vec<String>,
    /// Arguments to forward to custom commands
    #[arg(last = true)]
    arguments: Vec<String>,
}

/// Main entry point for `garden cmd <query> <command>...`.
pub fn main_cmd(app_context: &model::ApplicationContext, options: &CmdOptions) -> Result<()> {
    if app_context.options.debug_level("cmd") > 0 {
        debug!("query: {}", options.query);
        debug!("commands: {:?}", options.commands);
        debug!("arguments: {:?}", options.arguments);
    }
    let params = CmdParams::from_cmd_options(options);
    let exit_status = cmd(app_context, &options.query, &params)?;

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
    breadth_first: bool,
    keep_going: bool,
    exit_on_error: bool,
}

impl CmdParams {
    pub fn new() -> Self {
        Self {
            exit_on_error: true,
            ..CmdParams::default()
        }
    }

    /// Build CmdParams from a CmdOptions struct
    pub fn from_cmd_options(options: &CmdOptions) -> Self {
        let mut params = Self::new();
        params.commands = options.commands.clone();
        params.arguments = options.arguments.clone();
        params.breadth_first = options.breadth_first;
        params.exit_on_error = !options.no_errexit;
        params.keep_going = options.keep_going;

        params
    }

    /// Build CmdParams from a CustomOptions struct
    pub fn from_custom_options(options: &CustomOptions) -> Self {
        let mut params = CmdParams::new();
        // Add the custom command name to the list of commands. cmds() operates on a vec of commands.
        params.arguments = options.arguments.clone();
        params.queries = options.queries.clone();
        // Default to "." when no queries have been specified.
        if params.queries.is_empty() {
            params.queries.push(".".into());
        }

        // Custom commands run breadth-first. The distinction shouldn't make a difference in
        // practice because "garden <custom-cmd> ..." is only able to run a single command, but we
        // use breadth-first because it retains the original implementation/behavior from before
        // --breadth-first was added to "garden cmd" and made opt-in.
        params.breadth_first = true;
        params.keep_going = options.keep_going;
        params.exit_on_error = !options.no_errexit;

        params
    }
}

/// Format an error
fn format_error<I: CommandFactory>(err: clap::Error) -> clap::Error {
    let mut cmd = I::command();
    err.format(&mut cmd)
}

/// Main entry point for `garden <command> <query>...`.
pub fn main_custom(app_context: &model::ApplicationContext, arguments: &Vec<String>) -> Result<()> {
    // Set the command name to "garden <custom>".
    let name = &arguments[0];
    let garden_custom = format!("garden {name}");
    let cli = CustomOptions::command().bin_name(garden_custom);
    let matches = cli.get_matches_from(arguments);
    let options = <CustomOptions as FromArgMatches>::from_arg_matches(&matches)
        .map_err(format_error::<CustomOptions>)?;

    if app_context.options.debug_level("cmd") > 0 {
        debug!("command: {}", name);
        debug!("queries: {:?}", options.queries);
        debug!("arguments: {:?}", options.arguments);
    }

    let mut params = CmdParams::from_custom_options(&options);
    // Add the custom command name to the list of commands. cmds() operates on a vec of commands.
    params.commands.push(name.to_string());

    cmds(app_context, &params)
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
    app_context: &model::ApplicationContext,
    query: &str,
    params: &CmdParams,
) -> Result<i32> {
    // Mutable scope for app.get_root_config_mut()
    let config = app_context.get_root_config_mut();
    // Resolve the tree query into a vector of tree contexts.
    let contexts = query::resolve_trees(app_context, config, query);

    if params.breadth_first {
        run_cmd_breadth_first(app_context, &contexts, params)
    } else {
        run_cmd_depth_first(app_context, &contexts, params)
    }
}

pub fn run_cmd_breadth_first(
    app_context: &model::ApplicationContext,
    contexts: &[model::TreeContext],
    params: &CmdParams,
) -> Result<i32> {
    let mut exit_status: i32 = errors::EX_OK;
    let quiet = app_context.options.quiet;
    let verbose = app_context.options.verbose;
    let shell = app_context.get_root_config().shell.to_string();

    // Loop over each command, evaluate the tree environment,
    // and run the command in each context.
    for name in &params.commands {
        // One invocation runs multiple commands
        for context in contexts {
            // Skip symlink trees.
            let config = match context.config {
                Some(config_id) => app_context.get_config(config_id),
                None => app_context.get_root_config(),
            };
            let tree = match config.trees.get(&context.tree) {
                Some(tree) => tree,
                None => continue,
            };
            if tree.is_symlink {
                continue;
            }
            // Evaluate the tree environment
            let env = eval::environment(app_context, config, context);

            // Run each command in the tree's context
            let path = tree.path_as_ref()?.to_string();
            // Sparse gardens/missing trees are ok -> skip these entries.
            if !model::print_tree(tree, config.tree_branches, verbose, quiet) {
                continue;
            }

            // One command maps to multiple command sequences.
            // When the scope is tree, only the tree's commands
            // are included.  When the scope includes a gardens,
            // its matching commands are appended to the end.
            let cmd_seq_vec = eval::command(app_context, context, name);
            app_context.get_root_config_mut().reset();

            if let Err(cmd_status) = run_cmd_vec(
                &app_context.options,
                &path,
                &shell,
                &env,
                &cmd_seq_vec,
                &params.arguments,
                params.exit_on_error,
            ) {
                exit_status = cmd_status;
                if !params.keep_going {
                    return Ok(cmd_status);
                }
            }
        }
    }

    // Return the last non-zero exit status.
    Ok(exit_status)
}

pub fn run_cmd_depth_first(
    app_context: &model::ApplicationContext,
    contexts: &[model::TreeContext],
    params: &CmdParams,
) -> Result<i32> {
    let mut exit_status: i32 = errors::EX_OK;
    let quiet = app_context.options.quiet;
    let verbose = app_context.options.verbose;
    let shell = app_context.get_root_config().shell.to_string();

    // Loop over each context, evaluate the tree environment and run the command.
    for context in contexts {
        // Skip symlink trees.
        let config = match context.config {
            Some(config_id) => app_context.get_config(config_id),
            None => app_context.get_root_config(),
        };
        let tree = match config.trees.get(&context.tree) {
            Some(tree) => tree,
            None => continue,
        };
        if tree.is_symlink {
            continue;
        }
        // Evaluate the tree environment
        let env = eval::environment(app_context, config, context);
        // Run each command in the tree's context
        let path = tree.path_as_ref()?.to_string();
        // Sparse gardens/missing trees are ok -> skip these entries.
        if !model::print_tree(tree, config.tree_branches, verbose, quiet) {
            continue;
        }

        // One invocation runs multiple commands
        for name in &params.commands {
            // One command maps to multiple command sequences.
            // When the scope is tree, only the tree's commands
            // are included.  When the scope includes a gardens,
            // its matching commands are appended to the end.
            let cmd_seq_vec = eval::command(app_context, context, name);
            app_context.get_root_config_mut().reset();

            if let Err(cmd_status) = run_cmd_vec(
                &app_context.options,
                &path,
                &shell,
                &env,
                &cmd_seq_vec,
                &params.arguments,
                params.exit_on_error,
            ) {
                exit_status = cmd_status;
                if !params.keep_going {
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
    options: &cli::MainOptions,
    path: &str,
    shell: &str,
    env: &Vec<(String, String)>,
    cmd_seq_vec: &[Vec<String>],
    arguments: &[String],
    exit_on_error: bool,
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
            if exit_on_error {
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
                if exit_on_error {
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
pub fn cmds(app: &model::ApplicationContext, params: &CmdParams) -> Result<()> {
    let mut exit_status = errors::EX_OK;

    for query in &params.queries {
        let status = cmd(app, query, params).unwrap_or(errors::EX_IOERR);
        if status != errors::EX_OK {
            exit_status = status;
            if !params.keep_going {
                break;
            }
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}
