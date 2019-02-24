extern crate subprocess;

use ::cmd;
use ::config;
use ::eval;
use ::model;
use ::query;


/// garden cmd <tree-expr> <command-name>*
pub fn main(options: &mut model::CommandOptions) {
    options.args.insert(0, "garden cmd".to_string());

    let mut expr = String::new();
    let mut commands: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden cmd - run preset commands over gardens");

        ap.refer(&mut options.keep_going)
            .add_option(&["-k", "--keep-going"], argparse::StoreTrue,
                        "continue to the next tree when errors occur");

        ap.refer(&mut expr).required()
            .add_argument("tree-expr", argparse::Store,
                          "gardens/trees to exec (tree expression)");

        ap.refer(&mut commands).required()
            .add_argument("commands", argparse::List,
                          "commands to run over resolved trees");

        ap.stop_on_first_argument(true);
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let verbose = options.is_debug("config::new");
    let mut cfg = config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", cfg);
    }
    if options.is_debug("cmd") {
        debug!("subcommand: cmd");
        debug!("expr: {}", expr);
        debug!("commands: {:?}", commands);
    }

    let quiet = options.quiet;
    let verbose = options.verbose;
    let keep_going = options.keep_going;

    let exit_status = cmd(&mut cfg, quiet, verbose, keep_going, &expr, &commands);
    std::process::exit(exit_status);
}


/// garden <command-name> <tree-expr>*
pub fn custom(options: &mut model::CommandOptions, command: &String) {
    options.args.insert(0, "garden cmd".to_string());

    let mut exprs: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden cmd - run preset commands over gardens");

        ap.refer(&mut options.keep_going)
            .add_option(&["-k", "--keep-going"], argparse::StoreTrue,
                        "continue to the next tree when errors occur");

        ap.refer(&mut exprs).required()
            .add_argument("tree-exprs", argparse::List,
                          "gardens/trees to exec (tree expressions)");


        ap.stop_on_first_argument(true);
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let verbose = options.is_debug("config::new");
    let mut cfg = config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", cfg);
    }
    if options.is_debug("cmd") {
        debug!("command: {}", command);
        debug!("exprs: {:?}", exprs);
    }

    let quiet = options.quiet;
    let verbose = options.verbose;
    let keep_going = options.keep_going;

    let exit_status = cmds(&mut cfg, quiet, verbose, keep_going, command, &exprs);
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
    expr: &str,
    commands: &Vec<String>,
) -> i32 {
    // Resolve the tree expression into a vector of tree contexts.
    let contexts = query::resolve_trees(config, expr);
    let mut exit_status: i32 = 0;

    // Loop over each context, evaluate the tree environment,
    // and run the command.
    for context in &contexts {
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
                    let mut exec = subprocess::Exec::shell(&cmd_str).cwd(&path);
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


/// Run cmd() over a Vec of tree expressions
pub fn cmds(
    config: &mut model::Configuration,
    quiet: bool,
    verbose: bool,
    keep_going: bool,
    command: &str,
    exprs: &Vec<String>,
) -> i32 {
    let mut exit_status: i32 = 0;

    let mut commands: Vec<String> = Vec::new();
    commands.push(command.into());

    for expr in exprs {
        let status = cmd(config, quiet, verbose, keep_going, &expr, &commands);
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
