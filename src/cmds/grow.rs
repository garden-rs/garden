/// Grow garden worktrees
use std::collections::{HashMap, HashSet};

use anyhow::Result;
use clap::Parser;

use crate::{cmd, display, errors, eval, git, model, query};

type GitConfigMap = HashMap<String, HashSet<String>>;

/// Options for the "garden grow" command
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct GrowOptions {
    /// Filter trees by name post-query using a glob pattern
    #[arg(long, short, default_value = "*")]
    trees: String,
    /// Tree query for the gardens, groups or trees to grow
    #[arg(required = true)]
    queries: Vec<String>,
}

/// Main entry point for the "garden grow" command
pub fn main(app: &model::ApplicationContext, options: &GrowOptions) -> Result<()> {
    let quiet = app.options.quiet;
    let verbose = app.options.verbose;

    let mut exit_status = errors::EX_OK;
    let mut configured_worktrees: HashSet<String> = HashSet::new();
    for query in &options.queries {
        let status = grow(
            app,
            &mut configured_worktrees,
            quiet,
            verbose,
            query,
            &options.trees,
        )?;
        if status != errors::EX_OK {
            exit_status = status;
        }
    }

    // Return the last non-zero exit status.
    cmd::result_from_exit_status(exit_status).map_err(|err| err.into())
}

/// Create/update trees in the evaluated tree query.
fn grow(
    app_context: &model::ApplicationContext,
    configured_worktrees: &mut HashSet<String>,
    quiet: bool,
    verbose: u8,
    query: &str,
    tree_pattern: &str,
) -> Result<i32> {
    let config = app_context.get_root_config();
    let contexts = query::resolve_and_filter_trees(app_context, config, query, tree_pattern);
    let mut exit_status = errors::EX_OK;

    for ctx in &contexts {
        let status =
            grow_tree_from_context(app_context, configured_worktrees, ctx, quiet, verbose)?;
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
    app_context: &model::ApplicationContext,
    configured_worktrees: &mut HashSet<String>,
    context: &model::TreeContext,
    quiet: bool,
    verbose: u8,
) -> Result<i32> {
    let config = match context.config {
        Some(config_id) => app_context.get_config(config_id),
        None => app_context.get_root_config(),
    };
    let mut exit_status = errors::EX_OK;
    let tree = match config.trees.get(&context.tree) {
        Some(tree) => tree,
        None => return Ok(exit_status),
    };

    let path = tree.path_as_ref()?.clone();
    display::print_tree_details(tree, config.tree_branches, verbose, quiet);

    let pathbuf = std::path::PathBuf::from(&path);
    let parent = pathbuf.parent().ok_or_else(|| {
        errors::GardenError::AssertionError(format!("unable to get parent directory for {path}"))
    })?;
    std::fs::create_dir_all(parent)
        .map_err(|err| errors::GardenError::OSError(format!("unable to create {path}: {err}")))?;

    if pathbuf.exists() {
        return update_tree_from_context(
            app_context,
            configured_worktrees,
            context,
            &pathbuf,
            None,
            quiet,
            verbose,
        );
    }

    if tree.is_symlink {
        let status = grow_symlink(app_context, context).unwrap_or(errors::EX_IOERR);
        if status != errors::EX_OK {
            exit_status = status;
        }
        return Ok(exit_status);
    }

    if tree.is_worktree {
        return grow_tree_from_context_as_worktree(
            app_context,
            configured_worktrees,
            context,
            quiet,
            verbose,
        );
    }

    // The "url" field maps to the default remote.
    let url = match tree.remotes.get(&tree.default_remote) {
        Some(remote) => eval::tree_value(
            app_context,
            config,
            remote.get_expr(),
            &context.tree,
            context.garden.as_ref(),
        ),
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

    // "git clone --remote <name>" uses an alternatively-named remote instead of "origin".
    if tree.default_remote != "origin" {
        cmd.push("--origin");
        cmd.push(&tree.default_remote);
    }

    // "git clone --branch=name" clones the named branch.
    let branch = eval::tree_value(
        app_context,
        config,
        tree.branch.get_expr(),
        &context.tree,
        context.garden.as_ref(),
    );

    let branch_opt;
    if !branch.is_empty() && !tree.branches.contains_key(&branch) {
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
    let status = cmd::status(exec);
    if status != 0 {
        exit_status = status;
    }

    let status = update_tree_from_context(
        app_context,
        configured_worktrees,
        context,
        &pathbuf,
        Some(&branch),
        quiet,
        verbose,
    )?;
    if status != errors::EX_OK {
        exit_status = status;
    }
    Ok(exit_status)
}

/// Print a command from a list of arguments.
fn print_quoted_command(command: &[&str]) {
    let quoted_args = command
        .iter()
        .map(|arg| cmd::shell_quote(arg))
        .collect::<Vec<String>>();
    print_command_str(&quoted_args.join(" "));
}

/// Print a single command from a string.
fn print_command_str(cmd: &str) {
    println!(
        "{} {}",
        display::Color::cyan(":"),
        display::Color::green(cmd),
    )
}

/// Add remotes that do not already exist and synchronize .git/config values.
fn update_tree_from_context(
    app_context: &model::ApplicationContext,
    configured_worktrees: &mut HashSet<String>,
    ctx: &model::TreeContext,
    path: &std::path::Path,
    branch: Option<&str>,
    _quiet: bool,
    verbose: u8,
) -> Result<i32> {
    let config = match ctx.config {
        Some(config_id) => app_context.get_config(config_id),
        None => app_context.get_root_config(),
    };
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
    let shared_worktree_path = query::shared_worktree_path(app_context, config, ctx);
    if !configured_worktrees.insert(shared_worktree_path) {
        return Ok(exit_status);
    }

    // Gather existing remotes
    let mut existing_remotes = HashSet::new();
    {
        let command = ["git", "remote"];
        let exec = cmd::exec_in_dir(&command, path);
        if let Ok(output) = cmd::stdout_to_string(exec) {
            for line in output.lines() {
                existing_remotes.insert(String::from(line));
            }
        }
    }

    // The "default-remote" field is used to change the name of the default "origin" remote.
    if tree.default_remote != "origin" {
        set_gitconfig_value("checkout.defaultRemoteName", &tree.default_remote, path);
    }

    // Loop over remotes and add/update the git remote configuration.
    for (remote, var) in &tree.remotes {
        let url = eval::tree_value(
            app_context,
            config,
            var.get_expr(),
            &ctx.tree,
            ctx.garden.as_ref(),
        );

        if existing_remotes.contains(remote) {
            let remote_key = format!("remote.{remote}.url");
            let command = ["git", "config", remote_key.as_ref(), url.as_ref()];
            if verbose > 1 {
                print_command_str(&command.join(" "));
            }
            let exec = cmd::exec_in_dir(&command, path);
            let status = cmd::status(exec);
            if status != errors::EX_OK {
                exit_status = status;
            }
        } else {
            let command = ["git", "remote", "add", remote.as_ref(), url.as_ref()];
            if verbose > 1 {
                print_command_str(&command.join(" "));
            }
            let exec = cmd::exec_in_dir(&command, path);
            let status = cmd::status(exec);
            if status != errors::EX_OK {
                exit_status = status;
            }

            // git config remote.<name>.tagopt --no-tags
            let key = format!("remote.{}.tagopt", remote);
            let command = ["git", "config", key.as_ref(), "--no-tags"];
            if verbose > 1 {
                print_command_str(&command.join(" "));
            }
            let exec = cmd::exec_in_dir(&command, path);
            let status = cmd::status(exec);
            if status != errors::EX_OK {
                exit_status = status;
            }
        }
    }

    // Set gitconfig settings.
    let mut gitconfig_cache: GitConfigMap = GitConfigMap::new();
    for (var_name, variables) in &tree.gitconfig {
        let name = eval::tree_value(
            app_context,
            config,
            var_name,
            &ctx.tree,
            ctx.garden.as_ref(),
        );
        for var in variables {
            let value = match var.get_value() {
                Some(precomputed_value) => precomputed_value.to_string(),
                None => eval::tree_value(
                    app_context,
                    config,
                    var.get_expr(),
                    &ctx.tree,
                    ctx.garden.as_ref(),
                ),
            };
            let status = if variables.len() > 1 {
                // Multiple values are set using "git config --add <name> <value>"
                append_gitconfig_value(&name, &value, path, &mut gitconfig_cache)
            } else {
                // Single values are set directly using "git config <name> <value>".
                set_gitconfig_value(&name, &value, path)
            };
            if status != errors::EX_OK {
                exit_status = status;
            }
        }
    }

    // Create configured tracking branches.
    if !tree.branches.is_empty() {
        // Gather existing branches.
        let branches = git::branches(path);
        // Create all configured tracking branches.
        for (branch, expr) in &tree.branches {
            if !branches.contains(branch) {
                let remote_branch = eval::value(app_context, config, expr.get_expr());
                if !remote_branch.is_empty() {
                    let command = ["git", "branch", "--track", branch, remote_branch.as_str()];
                    let exec = cmd::exec_in_dir(&command, path);
                    let status = cmd::status(exec);
                    if status != errors::EX_OK {
                        exit_status = status;
                    }
                }
            }
        }
    }

    // Checkout the configured branch if we are creating the repository initially.
    if let Some(branch) = branch {
        if tree.branches.contains_key(branch) {
            let command = ["git", "checkout", branch];
            let exec = cmd::exec_in_dir(&command, path);
            let status = cmd::status(exec);
            if status != errors::EX_OK {
                exit_status = status;
            }
        }
    }

    Ok(exit_status)
}

/// Apply a "gitconfig" value in the specified directory.
fn append_gitconfig_value(
    name: &str,
    value: &str,
    path: &std::path::Path,
    config_map: &mut GitConfigMap,
) -> i32 {
    // If the config_map doesn't contain this variable then we need
    // to query git for the current values to avoid appending values
    // that are already present.
    let needs_cache = !config_map.contains_key(name);
    if needs_cache {
        let cmd = ["git", "config", "--get-all", name];
        let exec = cmd::exec_in_dir(&cmd, path);
        if let Ok(output) = cmd::stdout_to_string(exec) {
            let mut values = HashSet::new();
            for value in output.lines() {
                values.insert(value.to_string());
            }
            config_map.insert(name.to_string(), values);
        } else {
            config_map.insert(name.to_string(), HashSet::new());
        }
    }

    let mut status = errors::EX_OK;
    if let Some(values) = config_map.get_mut(name) {
        // Now that we've populated the config_map cache then we
        // can avoid running "git config --add <name> <value>".
        if !values.contains(value) {
            values.insert(value.to_string());
            let command = ["git", "config", "--add", name, value];
            let exec = cmd::exec_in_dir(&command, path);
            status = cmd::status(exec)
        }
    }

    status
}

/// Set a simple gitconfig value.
fn set_gitconfig_value(name: &str, value: &str, path: &std::path::Path) -> i32 {
    let command = ["git", "config", name, value];
    let exec = cmd::exec_in_dir(&command, path);

    cmd::status(exec)
}

/// Use "git worktree" to create a worktree.
/// Grow the parent worktree first and then create our worktree.
fn grow_tree_from_context_as_worktree(
    app_context: &model::ApplicationContext,
    configured_worktrees: &mut HashSet<String>,
    ctx: &model::TreeContext,
    quiet: bool,
    verbose: u8,
) -> Result<i32> {
    let config = match ctx.config {
        Some(config_id) => app_context.get_config(config_id),
        None => app_context.get_root_config(),
    };
    let mut exit_status = errors::EX_OK;
    let tree = match config.trees.get(&ctx.tree) {
        Some(tree) => tree,
        None => return Ok(exit_status),
    };
    let worktree = eval::tree_value(
        app_context,
        config,
        tree.worktree.get_expr(),
        &ctx.tree,
        ctx.garden.as_ref(),
    );
    let branch = eval::tree_value(
        app_context,
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

    exit_status = grow_tree_from_context(
        app_context,
        configured_worktrees,
        &parent_ctx,
        quiet,
        verbose,
    )?;
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
        // Read the upstream branch from tree.<tree>.branches.<branch> when configured.
        // Defaults to "<remote>/<branch>" when not configured.
        if let Some(expr) = tree.branches.get(&branch) {
            remote_branch = eval::value(app_context, config, expr.get_expr());
        } else {
            // The "default-remote" field is used to change the name of the default "origin" remote.
            let default_remote = tree.default_remote.to_string();
            remote_branch = format!("{default_remote}/{branch}");
        }
        if !remote_branch.is_empty() {
            cmd.push(&remote_branch);
        }
    }

    if verbose > 1 {
        print_quoted_command(&cmd);
    }
    let exec = cmd::exec_in_dir(&cmd, parent_path);
    exit_status = cmd::status(exec);
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
fn grow_symlink(app_context: &model::ApplicationContext, ctx: &model::TreeContext) -> Result<i32> {
    let config = match ctx.config {
        Some(config_id) => app_context.get_config(config_id),
        None => app_context.get_root_config(),
    };
    let tree = match config.trees.get(&ctx.tree) {
        Some(tree) => tree,
        None => return Ok(errors::EX_OK),
    };
    // Invalid usage: non-symlink.
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
    // Note: the parent directory was already created by the caller.
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
