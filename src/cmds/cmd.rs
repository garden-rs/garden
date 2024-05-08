use anyhow::Result;
use clap::{CommandFactory, FromArgMatches, Parser};
use derivative::Derivative;

use crate::{cli, cmd, constants, display, errors, eval, model, path, query, syntax};

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
    /// Filter trees by name post-query using a glob pattern
    #[arg(long, short, default_value = "*")]
    trees: String,
    /// Do not pass "-e" to the shell.
    /// Prevent the "errexit" shell option from being set. By default, the "-e" option
    /// is passed to the configured shell so that multi-line and multi-statement
    /// commands halt execution when the first statement with a non-zero exit code is
    /// encountered. This option has the effect of making multi-line and
    /// multi-statement commands run all statements even when an earlier statement
    /// returns a non-zero exit code.
    #[arg(long = "no-errexit", short = 'n', default_value_t = true, action = clap::ArgAction::SetFalse)]
    exit_on_error: bool,
    /// Run commands even when the tree does not exist.
    #[arg(long, short)]
    force: bool,
    /// Increase verbosity level (default: 0)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Do not pass "-o shwordsplit" to zsh.
    /// Prevent the "shwordsplit" shell option from being set when using zsh.
    /// The "-o shwordsplit" option is passed to zsh by default so that unquoted
    /// $variable expressions are subject to word splitting, just like other shells.
    /// This option disables this behavior.
    #[arg(long = "no-wordsplit", short = 'z', default_value_t = true, action = clap::ArgAction::SetFalse)]
    word_split: bool,
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
#[command(bin_name = constants::GARDEN)]
pub struct CustomOptions {
    /// Continue to the next tree when errors occur
    #[arg(long, short)]
    keep_going: bool,
    /// Filter trees by name post-query using a glob pattern
    #[arg(long, short, default_value = "*")]
    trees: String,
    /// Do not pass "-e" to the shell.
    /// Prevent the "errexit" shell option from being set. By default, the "-e" option
    /// is passed to the configured shell so that multi-line and multi-statement
    /// commands halt execution when the first statement with a non-zero exit code is
    /// encountered. This option has the effect of making multi-line and
    /// multi-statement commands run all statements even when an earlier statement
    /// returns a non-zero exit code.
    #[arg(long = "no-errexit", short = 'n', default_value_t = true, action = clap::ArgAction::SetFalse)]
    exit_on_error: bool,
    /// Run commands even when the tree does not exist.
    #[arg(long, short)]
    force: bool,
    /// Increase verbosity level (default: 0)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Do not pass "-o shwordsplit" to zsh.
    /// Prevent the "shwordsplit" shell option from being set when using zsh.
    /// The "-o shwordsplit" option is passed to zsh by default so that unquoted
    /// $variable expressions are subject to word splitting, just like other shells.
    /// This option disables this behavior.
    #[arg(long = "no-wordsplit", short = 'z', default_value_t = true, action = clap::ArgAction::SetFalse)]
    word_split: bool,
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
pub fn main_cmd(app_context: &model::ApplicationContext, options: &mut CmdOptions) -> Result<()> {
    if app_context.options.debug_level(constants::DEBUG_LEVEL_CMD) > 0 {
        debug!("query: {}", options.query);
        debug!("commands: {:?}", options.commands);
        debug!("arguments: {:?}", options.arguments);
        debug!("trees: {:?}", options.trees);
    }
    if !app_context.get_root_config().shell_exit_on_error {
        options.exit_on_error = false;
    }
    if !app_context.get_root_config().shell_word_split {
        options.word_split = false;
    }
    let params: CmdParams = options.clone().into();
    let exit_status = cmd(app_context, &options.query, &params)?;

    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}

/// CmdParams are used to control the execution of run_cmd_vec().
///
/// `garden cmd` and `garden <custom-cmd>` parse command line arguments into CmdParams.
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
pub struct CmdParams {
    commands: Vec<String>,
    arguments: Vec<String>,
    queries: Vec<String>,
    tree_pattern: glob::Pattern,
    breadth_first: bool,
    force: bool,
    keep_going: bool,
    #[derivative(Default(value = "true"))]
    exit_on_error: bool,
    verbose: u8,
    #[derivative(Default(value = "true"))]
    word_split: bool,
}

/// Build CmdParams from a CmdOptions struct.
impl From<CmdOptions> for CmdParams {
    fn from(options: CmdOptions) -> Self {
        Self {
            commands: options.commands.clone(),
            arguments: options.arguments.clone(),
            breadth_first: options.breadth_first,
            exit_on_error: options.exit_on_error,
            force: options.force,
            keep_going: options.keep_going,
            tree_pattern: glob::Pattern::new(&options.trees).unwrap_or_default(),
            verbose: options.verbose,
            word_split: options.word_split,
            ..Default::default()
        }
    }
}

/// Build CmdParams from a CustomOptions struct
impl From<CustomOptions> for CmdParams {
    fn from(options: CustomOptions) -> Self {
        let mut params = Self {
            // Add the custom command name to the list of commands. cmds() operates on a vec of commands.
            arguments: options.arguments.clone(),
            queries: options.queries.clone(),
            // Custom commands run breadth-first. The distinction shouldn't make a difference in
            // practice because "garden <custom-cmd> ..." is only able to run a single command, but we
            // use breadth-first because it retains the original implementation/behavior from before
            // --breadth-first was added to "garden cmd" and made opt-in.
            breadth_first: true,
            keep_going: options.keep_going,
            exit_on_error: options.exit_on_error,
            force: options.force,
            tree_pattern: glob::Pattern::new(&options.trees).unwrap_or_default(),
            verbose: options.verbose,
            word_split: options.word_split,
            ..Default::default()
        };

        // Default to "." when no queries have been specified.
        if params.queries.is_empty() {
            params.queries.push(constants::DOT.into());
        }

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

    let mut options = <CustomOptions as FromArgMatches>::from_arg_matches(&matches)
        .map_err(format_error::<CustomOptions>)?;
    if !app_context.get_root_config().shell_exit_on_error {
        options.exit_on_error = false;
    }
    if !app_context.get_root_config().shell_word_split {
        options.word_split = false;
    }

    if app_context.options.debug_level(constants::DEBUG_LEVEL_CMD) > 0 {
        debug!("command: {}", name);
        debug!("queries: {:?}", options.queries);
        debug!("arguments: {:?}", options.arguments);
        debug!("trees: {:?}", options.trees);
    }

    // Add the custom command name to the list of commands. cmds() operates on a vec of commands.
    let mut params: CmdParams = options.clone().into();
    params.commands.push(name.to_string());

    cmds(app_context, &params)
}

/// Strategy: resolve the trees down to a set of tree indexes paired with
/// an optional garden context.
///
/// If the names resolve to gardens, each garden is processed independently.
/// Trees that exist in multiple matching gardens will be processed multiple
/// times.
///
/// If the names resolve to trees, each tree is processed independently
/// with no garden context.

fn cmd(app_context: &model::ApplicationContext, query: &str, params: &CmdParams) -> Result<i32> {
    let config = app_context.get_root_config_mut();
    let contexts = query::resolve_trees(app_context, config, None, query);
    if params.breadth_first {
        run_cmd_breadth_first(app_context, &contexts, params)
    } else {
        run_cmd_depth_first(app_context, &contexts, params)
    }
}

/// Split the configured shell into command-line arguments.
fn shlex_split(shell: &str) -> Vec<String> {
    match shlex::split(shell) {
        Some(shell_command) if !shell_command.is_empty() => shell_command,
        _ => {
            vec![shell.to_string()]
        }
    }
}

/// Run commands breadth-first. Each command is run in all trees before running the next command.
fn run_cmd_breadth_first(
    app_context: &model::ApplicationContext,
    contexts: &[model::TreeContext],
    params: &CmdParams,
) -> Result<i32> {
    let mut exit_status: i32 = errors::EX_OK;
    let quiet = app_context.options.quiet;
    let verbose = app_context.options.verbose + params.verbose;
    let shell = app_context.get_root_config().shell.as_str();
    let shell_params = ShellParams::new(shell, params.exit_on_error, params.word_split);
    // Loop over each command, evaluate the tree environment,
    // and run the command in each context.
    for name in &params.commands {
        // One invocation runs multiple commands
        for context in contexts {
            // Skip filtered trees.
            if !params.tree_pattern.matches(&context.tree) {
                continue;
            }
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
            let Ok(mut path) = tree.path_as_ref() else {
                continue;
            };
            let fallback_path;
            // Sparse gardens/missing trees are ok -> skip these entries.
            if !display::print_tree(tree, config.tree_branches, verbose, quiet, params.force) {
                if params.force {
                    fallback_path = config.fallback_execdir_string();
                    path = &fallback_path;
                } else {
                    continue;
                }
            }

            // Expand one named command to include its pre-commands and post-commands.
            let command_names = cmd::expand_command_names(app_context, context, name);
            for command_name in command_names.iter() {
                // One command maps to multiple command sequences.
                // When the scope is tree, only the tree's commands
                // are included.  When the scope includes a gardens,
                // its matching commands are appended to the end.
                let cmd_seq_vec = eval::command(app_context, context, command_name);
                app_context.get_root_config_mut().reset();

                if let Err(cmd_status) = run_cmd_vec(
                    &app_context.options,
                    path,
                    &shell_params,
                    &env,
                    &cmd_seq_vec,
                    params,
                ) {
                    exit_status = cmd_status;
                    if !params.keep_going {
                        return Ok(cmd_status);
                    }
                }
            }
        }
    }

    // Return the last non-zero exit status.
    Ok(exit_status)
}

/// The configured shell state.
struct ShellParams {
    /// The shell string is parsed into command line arguments.
    shell_command: Vec<String>,
    /// Is this a shell script runner that requires $0 to be passed as the first argument?
    is_shell: bool,
}

impl ShellParams {
    fn new(shell: &str, exit_on_error: bool, word_split: bool) -> Self {
        let mut shell_command = shlex_split(shell);
        let basename = path::str_basename(&shell_command[0]);
        // Does the shell understand "-e" for errexit?
        let is_shell = path::is_shell(basename);
        let is_zsh = matches!(basename, constants::SHELL_ZSH);
        // Does the shell use "-e <string>" or "-c <string>" to evaluate commands?
        let is_dash_e = matches!(
            basename,
            constants::SHELL_BUN
                | constants::SHELL_NODE
                | constants::SHELL_NODEJS
                | constants::SHELL_PERL
                | constants::SHELL_RUBY
        );
        // Is the shell a full-blown command with "-c" and everything defined by the user?
        // If so we won't manage the custom shell options ourselves.
        let is_custom = shell_command.len() > 1;
        if !is_custom {
            if word_split && is_zsh {
                shell_command.push(string!("-o"));
                shell_command.push(string!("shwordsplit"));
            }
            if is_zsh {
                shell_command.push(string!("+o"));
                shell_command.push(string!("nomatch"));
            }
            if exit_on_error && is_shell {
                shell_command.push(string!("-e"));
            }
            if is_dash_e {
                shell_command.push(string!("-e"));
            } else {
                shell_command.push(string!("-c"));
            }
        }

        Self {
            shell_command,
            is_shell,
        }
    }

    /// Return ShellParams from a "#!" shebang line string.
    fn from_str(shell: &str) -> Self {
        let shell_command = shlex_split(shell);
        let basename = path::str_basename(&shell_command[0]);
        // Does the shell understand "-e" for errexit?
        let is_shell = path::is_shell(basename);

        Self {
            shell_command,
            is_shell,
        }
    }
}

/// Run commands depth-first. All commands are run in each tree before visiting the next tree.
fn run_cmd_depth_first(
    app_context: &model::ApplicationContext,
    contexts: &[model::TreeContext],
    params: &CmdParams,
) -> Result<i32> {
    let mut exit_status: i32 = errors::EX_OK;
    let quiet = app_context.options.quiet;
    let verbose = app_context.options.verbose + params.verbose;
    let shell = app_context.get_root_config().shell.as_str();
    let shell_params = ShellParams::new(shell, params.exit_on_error, params.word_split);
    // Loop over each context, evaluate the tree environment and run the command.
    for context in contexts {
        // Skip filtered trees.
        if !params.tree_pattern.matches(&context.tree) {
            continue;
        }
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
        let fallback_path;
        let Ok(mut path) = tree.path_as_ref() else {
            continue;
        };
        // Sparse gardens/missing trees are ok -> skip these entries.
        if !display::print_tree(tree, config.tree_branches, verbose, quiet, params.force) {
            if params.force {
                fallback_path = config.fallback_execdir_string();
                path = &fallback_path;
            } else {
                continue;
            }
        }
        // One invocation runs multiple commands
        for name in &params.commands {
            // Expand one named command to include its pre-commands and post-commands.
            let command_names = cmd::expand_command_names(app_context, context, name);
            for command_name in command_names.iter() {
                // One command maps to multiple command sequences.
                // When the scope is tree, only the tree's commands
                // are included.  When the scope includes a gardens,
                // its matching commands are appended to the end.
                let cmd_seq_vec = eval::command(app_context, context, command_name);
                app_context.get_root_config_mut().reset();
                if let Err(cmd_status) = run_cmd_vec(
                    &app_context.options,
                    path,
                    &shell_params,
                    &env,
                    &cmd_seq_vec,
                    params,
                ) {
                    exit_status = cmd_status;
                    if !params.keep_going {
                        return Ok(cmd_status);
                    }
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
    shell_params: &ShellParams,
    env: &Vec<(String, String)>,
    cmd_seq_vec: &[Vec<String>],
    params: &CmdParams,
) -> Result<(), i32> {
    // Get the current executable name
    let current_exe = cmd::current_exe();
    let mut exit_status = errors::EX_OK;
    for cmd_seq in cmd_seq_vec {
        for cmd_str in cmd_seq {
            if options.verbose > 1 {
                println!(
                    "{} {}",
                    display::Color::cyan(":"),
                    display::Color::green(&cmd_str),
                );
            }
            // Create a custom ShellParams when "#!" is used.
            let cmd_shell_params;
            let (cmd_str, shell_params) = match syntax::split_shebang(cmd_str) {
                Some((shell_cmd, cmd_str)) => {
                    cmd_shell_params = ShellParams::from_str(shell_cmd);
                    (cmd_str, &cmd_shell_params)
                }
                None => (cmd_str.as_str(), shell_params),
            };
            let mut exec = subprocess::Exec::cmd(&shell_params.shell_command[0]).cwd(path);
            exec = exec.args(&shell_params.shell_command[1..]);
            exec = exec.arg(cmd_str);
            if shell_params.is_shell {
                // Shells require $0 to be specified when using -c to run commands in order to make $1 and friends
                // behave intuitively from within the script. The garden executable's location is
                // provided in $0 for convenience.
                exec = exec.arg(current_exe.as_str());
            }
            exec = exec.args(&params.arguments);
            // Update the command environment
            for (k, v) in env {
                exec = exec.env(k, v);
            }
            // When a command list is used then the return code from the final command
            // is the one that is returned when --no-errexit is in effect.
            let status = cmd::status(exec);
            if status != errors::EX_OK {
                exit_status = status;
                if params.exit_on_error {
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
fn cmds(app: &model::ApplicationContext, params: &CmdParams) -> Result<()> {
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
