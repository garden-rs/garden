/// Grow garden worktrees
use super::super::cmd;
use super::super::errors;
use super::super::eval;
use super::super::model;
use super::super::query;

use anyhow::Result;
use clap::Parser;
use std::collections::HashSet;

/// Options for the "garden grow" command
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct GrowOptions {
    /// Tree query for the gardens, groups or trees to grow
    #[arg(required = true)]
    queries: Vec<String>,
}

/// Main entry point for the "garden grow" command
pub fn main(app: &mut model::ApplicationContext, options: &GrowOptions) -> Result<()> {
    let quiet = app.options.quiet;
    let verbose = app.options.verbose;

    let mut exit_status = errors::EX_OK;
    let mut configured_worktrees: HashSet<String> = HashSet::new();
    let config = app.get_root_config_mut();
    for query in &options.queries {
        let status = grow(config, &mut configured_worktrees, quiet, verbose, query)?;
        if status != errors::EX_OK {
            exit_status = status;
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}

/// Create/update trees in the evaluated tree query.
pub fn grow(
    config: &mut model::Configuration,
    configured_worktrees: &mut HashSet<String>,
    quiet: bool,
    verbose: u8,
    query: &str,
) -> Result<i32> {
    let contexts = query::resolve_trees(config, query);
    let mut exit_status = errors::EX_OK;

    for ctx in &contexts {
        let status = grow_tree_from_context(config, configured_worktrees, ctx, quiet, verbose)?;
        if status != errors::EX_OK {
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
    configured_worktrees: &mut HashSet<String>,
    ctx: &model::TreeContext,
    quiet: bool,
    verbose: u8,
) -> Result<i32> {
    let mut exit_status = errors::EX_OK;
    let tree = match config.trees.get(&ctx.tree) {
        Some(tree) => tree,
        None => return Ok(exit_status),
    };

    let path = tree.path_as_ref()?.clone();
    model::print_tree_details(tree, verbose, quiet);

    let pathbuf = std::path::PathBuf::from(&path);
    let parent = pathbuf.parent().ok_or_else(|| {
        errors::GardenError::AssertionError(format!("unable to get parent directory for {path}"))
    })?;
    std::fs::create_dir_all(parent)
        .map_err(|err| errors::GardenError::OSError(format!("unable to create {path}: {err}")))?;

    if pathbuf.exists() {
        return update_tree_from_context(
            config,
            configured_worktrees,
            ctx,
            &pathbuf,
            quiet,
            verbose,
        );
    }

    if tree.is_symlink {
        let status = grow_symlink(config, ctx).unwrap_or(errors::EX_IOERR);
        if status != errors::EX_OK {
            exit_status = status;
        }
        return Ok(exit_status);
    }

    if tree.is_worktree {
        return grow_tree_from_context_as_worktree(
            config,
            configured_worktrees,
            ctx,
            quiet,
            verbose,
        );
    }

    // The "origin" remote is cloned by convention. The "url" field maps to "origin".
    let url = match tree.remotes.get("origin") {
        Some(remote) => eval::tree_value(config, remote.get_expr(), &ctx.tree, ctx.garden.as_ref()),
        None => return Ok(exit_status),
    };

    // git clone [options] <url> <path>
    let mut cmd: Vec<&str> = ["git", "clone"].to_vec();

    // [options]
    //
    // "git clone --bare" clones bare repositories.
    if tree.is_bare_repository {
        cmd.push("--bare");
    }

    // "git clone --branch=name" clones the named branch.
    let branch_var = tree.branch.clone();
    let branch = eval::tree_value(
        config,
        branch_var.get_expr(),
        &ctx.tree,
        ctx.garden.as_ref(),
    );
    let branch_opt;
    if !branch.is_empty() {
        branch_opt = format!("--branch={branch}");
        cmd.push(&branch_opt);
    }
    // "git clone --depth=N" creates shallow clones with truncated history.
    let clone_depth = tree.clone_depth;
    let clone_depth_opt;
    if clone_depth > 0 {
        clone_depth_opt = format!("--depth={clone_depth}");
        cmd.push(&clone_depth_opt);
    }
    // "git clone --depth=N" clones a single branch by default.
    // We generally want all branches available in our clones so we default to
    // "single-branch: false" so that "--no-single-branch" is used. This makes
    // all branches available by default.
    let is_single_branch = tree.is_single_branch;
    if is_single_branch {
        cmd.push("--single-branch");
    } else {
        cmd.push("--no-single-branch");
    }

    // <url> <path>
    cmd.push(&url);
    cmd.push(&path);
    if verbose > 1 {
        print_quoted_command(&cmd);
    }

    let exec = cmd::exec_cmd(&cmd);
    let status = cmd::status(exec.join());
    if status != 0 {
        exit_status = status;
    }

    let status =
        update_tree_from_context(config, configured_worktrees, ctx, &pathbuf, quiet, verbose)?;
    if status != errors::EX_OK {
        exit_status = status;
    }
    Ok(exit_status)
}

/// Print a command that will be executed.
fn print_quoted_command(command: &[&str]) {
    let mut quoted_args: Vec<String> = Vec::new();
    for cmd in command {
        let quoted = shlex::quote(cmd);
        quoted_args.push(quoted.as_ref().to_string());
    }

    print_command_str(&quoted_args.join(" "));
}

/// Print a command that will be executed from a string.
fn print_command_str(cmd: &str) {
    println!("{} {}", model::Color::cyan(":"), model::Color::green(cmd),)
}

/// Add remotes that do not already exist and synchronize .git/config values.
fn update_tree_from_context(
    config: &model::Configuration,
    configured_worktrees: &mut HashSet<String>,
    ctx: &model::TreeContext,
    path: &std::path::Path,
    _quiet: bool,
    verbose: u8,
) -> Result<i32> {
    let mut exit_status = errors::EX_OK;
    let tree = match config.trees.get(&ctx.tree) {
        Some(tree) => tree,
        None => return Ok(exit_status),
    };

    // Existing symlinks require no further processing.
    if tree.is_symlink {
        return Ok(exit_status);
    }

    // Repositories created using "git worktree" share a common Git configuration
    // and only need to be configured once. Skip configuring the repository
    // if we've already processed it.
    let shared_worktree_path = query::shared_worktree_path(config, ctx);
    if !configured_worktrees.insert(shared_worktree_path) {
        return Ok(exit_status);
    }

    // Gather existing remotes
    let mut existing_remotes = HashSet::new();
    {
        let command = ["git", "remote"];
        let exec = cmd::exec_in_dir(&command, path);
        if let Ok(x) = cmd::capture_stdout(exec) {
            let output = cmd::trim_stdout(&x);
            for line in output.lines() {
                existing_remotes.insert(String::from(line));
            }
        }
    }

    // Loop over remotes and add/update the git remote configuration.
    for (remote, var) in &tree.remotes {
        let url = eval::tree_value(config, var.get_expr(), &ctx.tree, ctx.garden.as_ref());

        let exec = if existing_remotes.contains(remote) {
            let remote_key = format!("remote.{remote}.url");
            let command = ["git", "config", remote_key.as_ref(), url.as_ref()];
            if verbose > 1 {
                print_command_str(&command.join(" "));
            }
            cmd::exec_in_dir(&command, path)
        } else {
            let command = ["git", "remote", "add", remote.as_ref(), url.as_ref()];
            if verbose > 1 {
                print_command_str(&command.join(" "));
            }
            cmd::exec_in_dir(&command, path)
        };

        let status = cmd::status(exec.join());
        if status != errors::EX_OK {
            exit_status = status;
        }
    }

    // Set gitconfig settings
    for (var_name, var) in &tree.gitconfig {
        let name = eval::tree_value(config, var_name, &ctx.tree, ctx.garden.as_ref());
        let value = match var.get_value() {
            Some(precomputed_value) => precomputed_value.to_string(),
            None => eval::tree_value(config, var.get_expr(), &ctx.tree, ctx.garden.as_ref()),
        };
        let command = ["git", "config", name.as_ref(), value.as_ref()];
        let exec = cmd::exec_in_dir(&command, path);
        let status = cmd::status(exec.join());
        if status != errors::EX_OK {
            exit_status = status;
        }
    }

    Ok(exit_status)
}

/// Use "git worktree" to create a worktree.
/// Grow the parent worktree first and then create our worktree.
fn grow_tree_from_context_as_worktree(
    config: &model::Configuration,
    configured_worktrees: &mut HashSet<String>,
    ctx: &model::TreeContext,
    quiet: bool,
    verbose: u8,
) -> Result<i32> {
    let mut exit_status = errors::EX_OK;
    let tree = match config.trees.get(&ctx.tree) {
        Some(tree) => tree,
        None => return Ok(exit_status),
    };
    let worktree = eval::tree_value(
        config,
        tree.worktree.get_expr(),
        &ctx.tree,
        ctx.garden.as_ref(),
    );
    let branch = eval::tree_value(
        config,
        tree.branch.get_expr(),
        &ctx.tree,
        ctx.garden.as_ref(),
    );

    let parent_ctx =
        query::tree_from_name(config, &worktree, ctx.garden.as_ref(), ctx.group.as_ref())
            .ok_or_else(|| errors::GardenError::WorktreeNotFound {
                tree: tree.get_name().to_string(),
                worktree: worktree.clone(),
            })?;

    exit_status =
        grow_tree_from_context(config, configured_worktrees, &parent_ctx, quiet, verbose)?;
    if exit_status != 0 {
        return Err(errors::GardenError::WorktreeParentCreationError {
            tree: tree.get_name().into(),
            worktree,
        }
        .into());
    }

    let tree_path = tree.path_as_ref()?;
    let parent_tree = match config.trees.get(&parent_ctx.tree) {
        Some(parent_tree) => parent_tree,
        None => {
            return Err(errors::GardenError::WorktreeNotFound {
                tree: tree.get_name().to_string(),
                worktree,
            }
            .into())
        }
    };
    let parent_path = parent_tree.path_as_ref()?;

    let mut cmd: Vec<&str> = ["git", "worktree", "add"].to_vec();
    if !branch.is_empty() {
        cmd.push("--track");
        cmd.push("-b");
        cmd.push(&branch);
    }

    // The parent_path is the base path from which we'll execute "git worktree add".
    // Compute a relative path to the child.
    let relative_path_str;
    if let Some(relative_path) = pathdiff::diff_paths(tree_path, parent_path) {
        relative_path_str = relative_path.to_string_lossy().to_string();
        cmd.push(&relative_path_str);
    } else {
        cmd.push(tree_path);
    }

    let remote_branch;
    if !branch.is_empty() {
        // TODO: Support tree.<tree>.branches.<branch-name>.upstream
        // to generalize the remote branch name instead of hard-coding "origin/".
        remote_branch = format!("origin/{branch}");
        cmd.push(&remote_branch);
    }

    if verbose > 1 {
        print_quoted_command(&cmd);
    }
    let exec = cmd::exec_in_dir(&cmd, parent_path);
    exit_status = cmd::status(exec.join());

    if exit_status != 0 {
        return Err(errors::GardenError::WorktreeGitCheckoutError {
            tree: tree.get_name().clone(),
            status: exit_status,
        }
        .into());
    }

    Ok(exit_status)
}

/// Initialize a tree symlink entry.
fn grow_symlink(config: &model::Configuration, ctx: &model::TreeContext) -> Result<i32> {
    let tree = match config.trees.get(&ctx.tree) {
        Some(tree) => tree,
        None => return Ok(errors::EX_OK),
    };
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
        .ok_or_else(|| errors::GardenError::AssertionError(format!("parent() failed: {path:?}")))?
        .to_path_buf();

    // Is the link target a child of the link's parent directory?
    let target = if symlink.starts_with(&parent) && symlink.strip_prefix(&parent).is_ok() {
        // If so, create the symlink using a relative path.
        symlink.strip_prefix(&parent)?.to_string_lossy()
    } else {
        // Use an absolute path otherwise.
        symlink.to_string_lossy()
    }
    .to_string();
    let target_path = std::path::PathBuf::from(&target);
    std::os::unix::fs::symlink(target_path, &path)?;

    Ok(errors::EX_OK)
}
