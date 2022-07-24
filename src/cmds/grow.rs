use anyhow::Result;

use super::super::cmd;
use super::super::errors;
use super::super::eval;
use super::super::model;
use super::super::query;

/// Main entry point for the "garden exec" command
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let mut queries = Vec::new();
    parse_args(&mut queries, &mut app.options);

    let quiet = app.options.quiet;
    let verbose = app.options.verbose;

    let mut exit_status = errors::EX_OK;
    let config = app.get_root_config_mut();
    for query in &queries {
        let status = grow(config, quiet, verbose, query)?;
        if status != errors::EX_OK {
            exit_status = status;
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}

/// Parse "garden grow" arguments.
fn parse_args(queries: &mut Vec<String>, options: &mut model::CommandOptions) {
    options.args.insert(0, "garden grow".into());

    let mut ap = argparse::ArgumentParser::new();
    ap.set_description("garden grow - Create and update gardens");

    ap.refer(queries).required().add_argument(
        "queries",
        argparse::List,
        "gardens/groups/trees to grow (tree queries)",
    );

    if let Err(err) = ap.parse(
        options.args.to_vec(),
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    ) {
        std::process::exit(err);
    }
}

/// Create/update trees in the evaluated tree query.
pub fn grow(
    config: &mut model::Configuration,
    quiet: bool,
    verbose: bool,
    query: &str,
) -> Result<i32> {
    let contexts = query::resolve_trees(config, query);
    let mut exit_status: i32 = 0;

    for ctx in &contexts {
        let status = grow_tree_from_context(config, ctx, quiet, verbose)?;
        if status != 0 {
            // Return the last non-zero exit status.
            exit_status = status;
        }
    }

    Ok(exit_status)
}

/// Grow the tree specified by the context into existence.
/// Trees without remotes are silently ignored.
fn grow_tree_from_context(
    config: &model::Configuration,
    ctx: &model::TreeContext,
    quiet: bool,
    verbose: bool,
) -> Result<i32> {
    let mut exit_status: i32 = 0;

    let path = config.trees[ctx.tree].path_as_ref()?.clone();
    model::print_tree_details(&config.trees[ctx.tree], verbose, quiet);

    let pathbuf = std::path::PathBuf::from(&path);
    if !pathbuf.exists() {
        let parent = pathbuf.parent().ok_or_else(|| {
            errors::GardenError::AssertionError(format!(
                "unable to get parent directory for {}",
                path
            ))
        })?;

        std::fs::create_dir_all(&parent).map_err(|err| {
            errors::GardenError::OSError(format!("unable to create {}: {}", path, err))
        })?;

        if config.trees[ctx.tree].is_symlink {
            let status = init_symlink(config, ctx).unwrap_or(errors::EX_IOERR);
            if status != errors::EX_OK {
                exit_status = status;
            }
            return Ok(exit_status);
        }

        if config.trees[ctx.tree].remotes.is_empty() {
            return Ok(exit_status);
        }

        // The first remote is "origin" by convention
        let remote = config.trees[ctx.tree].remotes[0].clone();
        let url = eval::tree_value(config, remote.get_expr(), ctx.tree, ctx.garden);

        // git clone [options] <url> <path>
        let mut command: Vec<String> = vec!["git".into(), "clone".into()];

        // [options]
        //
        // "git clone --branch=name" clones the named branch.
        let branch_var = config.trees[ctx.tree].branch.clone();
        let branch = eval::tree_value(config, branch_var.get_expr(), ctx.tree, ctx.garden);
        if !branch.is_empty() {
            command.push(format!("--branch={}", branch));
        }
        // "git clone --depth=N" creates shallow clones with truncated history.
        let clone_depth = config.trees[ctx.tree].clone_depth;
        if clone_depth > 0 {
            command.push(format!("--depth={}", clone_depth));
        }
        // "git clone --depth=N" clones a single branch by default.
        // We generally want all branches available in our clones so we default to
        // "single-branch: false" so that "--no-single-branch" is used. This makes
        // all branches available by default.
        let is_single_branch = config.trees[ctx.tree].is_single_branch;
        if is_single_branch {
            command.push("--single-branch".into());
        } else {
            command.push("--no-single-branch".into());
        }

        // <url> <path>
        command.push(url);
        command.push(path.to_string());

        let exec = cmd::exec_cmd(&command);
        let status = cmd::status(exec.join());
        if status != 0 {
            exit_status = status;
        }
    }

    // Existing symlinks require no further processing.
    if config.trees[ctx.tree].is_symlink {
        return Ok(exit_status);
    }

    // Loop over remotes, update them as needed
    let mut config_remotes = std::collections::HashMap::new();
    {
        // Immutable config scope
        for remote in &config.trees[ctx.tree].remotes {
            config_remotes.insert(
                String::from(remote.get_name()),
                String::from(remote.get_expr()),
            );
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
                existing_remotes.insert(String::from(line));
            }
        }
    }

    // Add/update git remote configuration.
    for (k, v) in &config_remotes {
        let url = eval::tree_value(config, v, ctx.tree, ctx.garden);

        let exec;
        if existing_remotes.contains(k) {
            let remote_key = format!("remote.{}.url", k);
            let command = ["git", "config", remote_key.as_ref(), url.as_ref()];
            exec = cmd::exec_in_dir(&command, &path);
        } else {
            let command = ["git", "remote", "add", k.as_ref(), url.as_ref()];
            exec = cmd::exec_in_dir(&command, &path);
        }
        let status = cmd::status(exec.join());
        if status != 0 {
            exit_status = status;
        }
    }

    // Set gitconfig settings
    let mut gitconfig = Vec::new();
    for cfg in &config.trees[ctx.tree].gitconfig {
        gitconfig.push(cfg.clone());
    }

    for var in &gitconfig {
        let value = eval::tree_value(config, var.get_expr(), ctx.tree, ctx.garden);
        let command = ["git", "config", var.get_name(), value.as_ref()];
        let exec = cmd::exec_in_dir(&command, &path);
        let status = cmd::status(exec.join());
        if status != 0 {
            exit_status = status;
        }
    }

    Ok(exit_status)
}

/// Initialize a tree symlink entry.

fn init_symlink(config: &model::Configuration, ctx: &model::TreeContext) -> Result<i32> {
    let tree = &config.trees[ctx.tree];
    // Invalid usage: non-symlink
    if !tree.is_symlink || tree.path_as_ref()?.is_empty() || tree.symlink_as_ref()?.is_empty() {
        return Err(errors::GardenError::ConfigurationError(format!(
            "invalid symlink: {}",
            tree.get_name()
        ))
        .into());
    }
    let path_str = tree.path_as_ref()?;
    let path = std::path::PathBuf::from(&path_str);

    // Leave existing symlinks as-is.
    if std::fs::read_link(&path).is_ok() || path.exists() {
        return Ok(errors::EX_OK);
    }

    let symlink_str = tree.symlink_as_ref()?;
    let symlink = std::path::PathBuf::from(&symlink_str);

    // Note: parent directory was already created by the caller.
    let parent = path
        .parent()
        .as_ref()
        .ok_or_else(|| errors::GardenError::AssertionError(format!("parent() failed: {:?}", path)))?
        .to_path_buf();

    // Is the link target a child of the link's parent directory?
    let target: String;
    if symlink.starts_with(&parent) && symlink.strip_prefix(&parent).is_ok() {
        // If so, create the symlink using a relative path.
        target = symlink.strip_prefix(&parent)?.to_string_lossy().into();
    } else {
        // Use an absolute path otherwise.
        target = symlink.to_string_lossy().into();
    }

    std::os::unix::fs::symlink(&target, &path)?;

    Ok(errors::EX_OK)
}
