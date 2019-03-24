use ::cmd;
use ::eval;
use ::model;
use ::query;


/// Main entry point for the "garden exec" command
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(app: &mut model::ApplicationContext) {
    let options = &mut app.options;
    let mut queries = Vec::new();
    parse_args(&mut queries, options);

    let quiet = options.quiet;
    let verbose = options.verbose;

    let mut exit_status = 0;
    for query in &queries {
        let status = init(&mut app.config, quiet, verbose, &query);
        if status != 0 {
            exit_status = status;
        }
    }
    std::process::exit(exit_status);
}

fn parse_args(queries: &mut Vec<String>, options: &mut model::CommandOptions) {
    // Parse arguments
    options.args.insert(0, "garden init".to_string());

    let mut ap = argparse::ArgumentParser::new();
    ap.set_description(
        "garden init - Create gardens or reinitialize existing ones");

    ap.refer(queries).required()
        .add_argument("queries", argparse::List,
                      "gardens/groups/trees to initialize (tree queries)");

    if let Err(err) = ap.parse(options.args.to_vec(),
                               &mut std::io::stdout(),
                               &mut std::io::stderr()) {
        std::process::exit(err);
    }
}


/// Execute a command over every tree in the evaluated tree query.
pub fn init(
    config: &mut model::Configuration,
    quiet: bool,
    verbose: bool,
    query: &str,
) -> i32 {
    let contexts = query::resolve_trees(config, query);
    let mut exit_status: i32 = 0;

    for ctx in &contexts {
        let name = config.trees[ctx.tree].name.to_string();
        let path = config.trees[ctx.tree].path.value.as_ref().unwrap().to_string();

        if !quiet {
            if verbose {
                eprintln!("# {}: {}", name, path);
            } else {
                eprintln!("# {}", name);
            }
        }

        let pathbuf = std::path::PathBuf::from(&path);
        if !pathbuf.exists() {
            let parent = match pathbuf.parent() {
                Some(result) => result,
                None => {
                    error!("unable to create parent directory for '{}'", path);
                }
            };

            if let Err(err) = std::fs::create_dir_all(&parent) {
                error!("unable to create '{}': {}", path, err);
            }

            if config.trees[ctx.tree].is_symlink {
                let status = init_symlink(config, ctx);
                if status != 0 {
                    exit_status = status;
                }
                continue;
            }

            if config.trees[ctx.tree].remotes.is_empty() {
                continue;
            }

            // The first remote is "origin" by convention
            let remote = config.trees[ctx.tree].remotes[0].clone();
            let url = eval::tree_value(config, &remote.expr,
                                       ctx.tree, ctx.garden);

            let command = ["git", "clone", url.as_ref(), path.as_ref()];
            let exec = cmd::exec_cmd(&command);
            let status = cmd::status(exec.join());
            if status != 0 {
                exit_status = status as i32;
            }
        }

        // Existing symlinks require no further processing.
        if config.trees[ctx.tree].is_symlink {
            continue;
        }

        // Loop over remotes, update them as needed
        let mut config_remotes = std::collections::HashMap::new();
        {
            // Immutable config scope
            for remote in &config.trees[ctx.tree].remotes {
                config_remotes.insert(remote.name.to_string(),
                                      remote.expr.to_string());
            }
        }

        // Gather existing remotes
        let mut existing_remotes = std::collections::HashSet::new();
        {
            let command = ["git", "remote"];
            let exec = cmd::exec_in_dir(&command, &path);
            if let Ok(x) = cmd::capture_stdout(exec) {
                let output = cmd::trim_stdout(&x);
                for line in output.lines() {
                    existing_remotes.insert(line.to_string());
                }
            }
        }

        // Add/update git remote configuration.
        for (k, v) in &config_remotes {
            let url = eval::tree_value(config, &v, ctx.tree, ctx.garden);

            let exec;
            if existing_remotes.contains(k) {
                let remote_key = format!("remote.{}.url", k);
                let command = [
                    "git", "config", remote_key.as_ref(), url.as_ref(),
                ];
                exec = cmd::exec_in_dir(&command, &path);
            } else {
                let command = [
                    "git", "remote", "add", k.as_ref(), url.as_ref(),
                ];
                exec = cmd::exec_in_dir(&command, &path);
            }
            let status = cmd::status(exec.join());
            if status != 0 {
                exit_status = status as i32;
            }
        }

        // Set gitconfig settings
        let mut gitconfig = Vec::new();
        for cfg in &config.trees[ctx.tree].gitconfig {
            gitconfig.push(cfg.clone());
        }

        for var in &gitconfig {
            let value = eval::tree_value(config, &var.expr,
                                         ctx.tree, ctx.garden);
            let command = [
                "git", "config", var.name.as_ref(), value.as_ref(),
            ];
            let exec = cmd::exec_in_dir(&command, &path);
            let status = cmd::status(exec.join());
            if status != 0 {
                exit_status = status as i32;
            }
        }
    }

    // Return the last non-zero exit status.
    exit_status
}


/// Initialize a tree symlink entry.

fn init_symlink(
    config: &model::Configuration,
    ctx: &model::TreeContext,
) -> i32 {
    let tree = &config.trees[ctx.tree];
    // Invalid usage: non-symlink
    if !tree.is_symlink
        || tree.path.value.is_none()
        || tree.path.value.as_ref().unwrap().is_empty()
        || tree.symlink.value.is_none()
        || tree.symlink.value.as_ref().unwrap().is_empty() {
        return 1;
    }

    let path_str = tree.path.value.as_ref().unwrap();
    let path = std::path::PathBuf::from(&path_str);

    // Leave existing paths as-is.
    if std::fs::read_link(&path).is_ok() || path.exists() {
        return 0;
    }

    let symlink_str = tree.symlink.value.as_ref().unwrap();
    let symlink = std::path::PathBuf::from(&symlink_str);

    // Note: parent directory was already created by the caller.
    let parent = path.parent().as_ref().unwrap().to_path_buf();

    // Is the link target a child of the link's parent directory?
    let target;
    if symlink.starts_with(&parent) && symlink.strip_prefix(&parent).is_ok() {
        // If so, create the symlink using a relative path.
        target = symlink.strip_prefix(&parent)
                        .unwrap().to_string_lossy().to_string();
    } else {
        // Use an absolute path otherwise.
        target = symlink.to_string_lossy().to_string();
    }

    if std::os::unix::fs::symlink(&target, &path).is_ok() {
        0
    } else {
        1
    }
}
