extern crate subprocess;

use super::command;
use super::eval;
use super::model;
use super::query;

/// Resolve garden and tree names into a set of trees
/// Strategy: resolve the trees down to a set of tree indexes paired with an
/// an optional garden context.
///
/// If the names resolve to gardens, each garden is processed independently.
/// Trees that exist in multiple matching gardens will be processed multiple
/// times.
///
/// If the names resolve to trees, each tree is processed independently
/// with no garden context.

pub fn main<S>(
    config: &mut model::Configuration,
    quiet: bool,
    verbose: bool,
    keep_going: bool,
    expr: S,
    commands: &Vec<String>,
) where S: Into<String> {

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
            let cmd_seq_vec = eval::command(config, context, name.to_string());
            config.reset();
            for cmd_seq in &cmd_seq_vec {
                for cmd in cmd_seq {
                    let mut exec = subprocess::Exec::shell(&cmd).cwd(&path);
                    // Update the command environment
                    for (k, v) in &env {
                        exec = exec.env(k, v);
                    }
                    let status = command::status(exec.join());
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
    std::process::exit(exit_status);
}
