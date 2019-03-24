extern crate subprocess;

use ::cmd;
use ::eval;
use ::model;
use ::query;


/// gdn cmd <query> <command>...
pub fn main(app: &mut model::ApplicationContext) {
    let config = &mut app.config;
    let options = &mut app.options;
    let mut query = String::new();
    let mut commands_and_args: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.silence_double_dash(false);
        ap.set_description("gdn cmd - run custom commands over gardens");

        ap.refer(&mut options.keep_going)
            .add_option(&["-k", "--keep-going"], argparse::StoreTrue,
                        "continue to the next tree when errors occur");

        ap.refer(&mut query).required()
            .add_argument("query", argparse::Store,
                          "gardens/groups/trees to exec (tree query)");

        ap.refer(&mut commands_and_args).required()
            .add_argument("commands", argparse::List,
                          "commands to run over resolved trees");

        options.args.insert(0, "gdn cmd".to_string());
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    if options.is_debug("cmd") {
        debug!("subcommand: cmd");
        debug!("query: {}", query);
        debug!("commands_and_args: {:?}", commands_and_args);
    }

    // Queries and arguments are separated by a double-dash "--" marker.
    let mut commands = Vec::new();
    let mut arguments = Vec::new();
    cmd::split_on_dash(&commands_and_args, &mut commands, &mut arguments);

    if options.is_debug("cmd") {
        debug!("commands: {:?}", commands);
        debug!("arguments: {:?}", arguments);
    }

    let quiet = options.quiet;
    let verbose = options.verbose;
    let keep_going = options.keep_going;

    let exit_status = cmd(config, quiet, verbose, keep_going, &query,
                          &commands, &arguments);
    std::process::exit(exit_status);
}


/// gdn <command> <query>...
pub fn custom(app: &mut model::ApplicationContext, command: &str) {
    let config = &mut app.config;
    let options = &mut app.options;
    let mut queries_and_arguments: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.silence_double_dash(false);
        ap.set_description("gdn cmd - run custom commands over gardens");

        ap.refer(&mut options.keep_going)
            .add_option(&["-k", "--keep-going"], argparse::StoreTrue,
                        "continue to the next tree when errors occur");

        ap.refer(&mut queries_and_arguments).required()
            .add_argument("queries", argparse::List,
                          "gardens/groups/trees to exec (tree queries)");


        options.args.insert(0, "gdn cmd".to_string());
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    if options.is_debug("cmd") {
        debug!("command: {}", command);
        debug!("queries_and_arguments: {:?}", queries_and_arguments);
    }
    let quiet = options.quiet;
    let verbose = options.verbose;
    let keep_going = options.keep_going;

    // Queries and arguments are separated by a double-dash "--" marker.
    let mut queries = Vec::new();
    let mut arguments = Vec::new();
    cmd::split_on_dash(&queries_and_arguments, &mut queries, &mut arguments);

    if options.is_debug("cmd") {
        debug!("queries {:?}", queries);
        debug!("arguments: {:?}", arguments);
    }

    let exit_status = cmds(config, quiet, verbose, keep_going,
                           command, &queries, &arguments);
    std::process::exit(exit_status);
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
    config: &mut model::Configuration,
    quiet: bool,
    verbose: bool,
    keep_going: bool,
    query: &str,
    commands: &Vec<String>,
    arguments: &Vec<String>,
) -> i32 {
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
        // Evaluate the tree environment
        let env = eval::environment(config, context);
        let mut path;

        // Run each command in the tree's context
        {
            let tree = &config.trees[context.tree];
            path = tree.path.value.as_ref().unwrap().to_string();
            // Sparse gardens/missing trees are ok -> skip these entries.
            if !std::path::PathBuf::from(&path).exists() {
                if !quiet {
                    if verbose {
                        eprintln!("# {}: {} (skipped)", tree.name,
                                  tree.path.value.as_ref().unwrap());
                    } else {
                        eprintln!("# {} (skipped)", tree.name);
                    }
                }
                continue;
            }
            if !quiet {
                if verbose {
                    eprintln!("# {}: {}", tree.name,
                              tree.path.value.as_ref().unwrap());
                } else {
                    eprintln!("# {}", tree.name);
                }
            }
        }

        // The "error" flag is set when a non-zero exit status is returned.
        let mut error = false;

        // Get the current executable name
        let current_exe;
        let current_exe_result = std::env::current_exe();
        if current_exe_result.is_ok() {
            current_exe =
                current_exe_result.as_ref().unwrap()
                .to_string_lossy().to_string();
        } else {
            current_exe = "gdn".to_string();
        }

        // One invocation runs multiple commands
        for name in commands {
            // One command maps to multiple command sequences.
            // When the scope is tree, only the tree's commands
            // are included.  When the scope includes a gardens,
            // its matching commands are appended to the end.
            error = false;
            let cmd_seq_vec = eval::command(config, context, &name);
            config.reset();
            for cmd_seq in &cmd_seq_vec {
                for cmd_str in cmd_seq {
                    let mut exec =
                        subprocess::Exec::shell(&cmd_str)
                        .arg(&current_exe)
                        .args(arguments)
                        .cwd(&path);
                    // Update the command environment
                    for (k, v) in &env {
                        exec = exec.env(k, v);
                    }
                    let status = cmd::status(exec.join());
                    if status != 0 {
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
    exit_status
}


/// Run cmd() over a Vec of tree queries
pub fn cmds(
    config: &mut model::Configuration,
    quiet: bool,
    verbose: bool,
    keep_going: bool,
    command: &str,
    queries: &Vec<String>,
    arguments: &Vec<String>,
) -> i32 {
    let mut exit_status: i32 = 0;

    let mut commands: Vec<String> = Vec::new();
    commands.push(command.into());

    for query in queries {
        let status = cmd(config, quiet, verbose, keep_going,
                         &query, &commands, arguments);
        if status != 0 {
            exit_status = status;
            if !keep_going {
                break;
            }
        }
    }

    // Return the last non-zero exit status.
    exit_status
}
