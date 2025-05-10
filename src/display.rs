use crate::{eval, git, model};
use yansi::Paint;

#[derive(Debug, Default, Clone, Copy)]
pub struct DisplayOptions {
    pub branches: bool,
    pub commands: bool,
    pub force: bool,
    pub quiet: bool,
    pub remotes: bool,
    pub verbose: u8,
    pub worktrees: bool,
}

pub(crate) fn display_missing_tree(
    tree: &model::Tree,
    path: &str,
    verbose: u8,
    force: bool,
) -> String {
    let skipped = if force {
        String::new()
    } else {
        " (skipped)".bold().to_string()
    };
    if verbose > 0 {
        format!(
            "{} {} {}{}",
            "#".black().bold(),
            tree.get_name().black().bold(),
            path.black().bold(),
            skipped
        )
    } else {
        format!(
            "{} {}{}",
            "#".black().bold(),
            tree.get_name().black().bold(),
            skipped
        )
    }
}

pub(crate) fn display_tree(
    tree: &model::Tree,
    path_str: &str,
    tree_branches: bool,
    verbose: u8,
) -> String {
    if verbose > 0 {
        if tree_branches {
            if let Some(path) = tree.canonical_pathbuf() {
                if let Some(branch) = git::branch(&path) {
                    return format!(
                        "{} {} {}{}{} {}",
                        "#".cyan(),
                        tree.get_name().blue().bold(),
                        "[".blue(),
                        branch.green().bold(),
                        "]".blue(),
                        path_str.blue()
                    );
                }
            }
        }
        format!(
            "{} {} {}",
            "#".cyan(),
            tree.get_name().blue().bold(),
            path_str.blue()
        )
    } else {
        if tree_branches {
            if let Some(path) = tree.canonical_pathbuf() {
                if let Some(branch) = git::branch(&path) {
                    return format!(
                        "{} {} {}{}{}",
                        "#".cyan(),
                        tree.get_name().blue().bold(),
                        "[".blue(),
                        branch.green().bold(),
                        "]".blue()
                    );
                }
            }
        }
        format!("{} {}", "#".cyan(), tree.get_name().blue().bold())
    }
}

/// Print a tree if it exists, otherwise print a missing tree
pub(crate) fn print_tree(tree: &model::Tree, options: &DisplayOptions) -> bool {
    if let Ok(path) = tree.path_as_ref() {
        // Sparse gardens/missing trees are expected. Skip these entries.
        if !std::path::PathBuf::from(&path).exists() {
            if !options.quiet {
                eprintln!(
                    "{}",
                    display_missing_tree(tree, path, options.verbose, options.force)
                );
            }
            return false;
        }

        print_tree_details(tree, options.branches, options.verbose, options.quiet);
        return true;
    }
    if !options.quiet {
        eprintln!(
            "{}",
            display_missing_tree(tree, "(invalid-path)", options.verbose, options.force)
        );
    }

    false
}

/// Print a tree.
pub(crate) fn print_tree_details(
    tree: &model::Tree,
    tree_branches: bool,
    verbose: u8,
    quiet: bool,
) {
    if quiet {
        return;
    }
    if let Ok(path) = tree.path_as_ref() {
        eprintln!("{}", display_tree(tree, path, tree_branches, verbose));
    }
}

/// Print non-grown / missing tree.
pub(crate) fn print_missing_tree(tree: &model::Tree, path: &str, verbose: u8) {
    if verbose > 0 {
        println!(
            "{} {} {}",
            "#-".red().dim(),
            tree.get_name().red(),
            path.red().dim()
        );
    } else {
        println!("{} {}", "#-".red().dim(), tree.get_name().red());
    }
}

/// Print a symlink tree entry.
pub(crate) fn print_symlink_tree_entry(tree: &model::Tree, path: &str, verbose: u8) {
    let symlink = match tree.symlink_as_ref() {
        Ok(symlink) => symlink,
        Err(_) => return,
    };
    if verbose > 0 {
        println!(
            "{} {} {} {} {}",
            "#+".cyan(),
            tree.get_name().blue().bold(),
            path.green(),
            "->".green(),
            symlink.yellow()
        );
    } else {
        println!(
            "{} {} {} {}",
            "#".cyan(),
            tree.get_name().blue().bold(),
            "->".green(),
            symlink.yellow()
        );
    }
}

/// Print the description, url, remotes and links for a tree
pub(crate) fn print_tree_extended_details(
    app_context: &model::ApplicationContext,
    context: &model::TreeContext,
    tree: &model::Tree,
    display_options: &DisplayOptions,
) {
    let config = match context.config {
        Some(config_id) => app_context.get_config(config_id),
        None => app_context.get_root_config(),
    };
    if !tree.description.is_empty() {
        println!("{}", tree.description.green());
    }
    if tree.is_worktree && !display_options.worktrees {
        return;
    }
    if display_options.remotes && !tree.remotes.is_empty() {
        println!("{}", "remotes:".blue());
        for (name, remote) in &tree.remotes {
            let value = eval::tree_variable(
                app_context,
                config,
                None,
                &context.tree,
                context.garden.as_ref(),
                remote,
            );
            println!("  {}{} {}", name.blue(), ":".blue(), value.yellow());
        }
    }
    if !tree.links.is_empty() {
        println!("{}", "links:".blue());
        for link in &tree.links {
            let value = eval::tree_variable(
                app_context,
                config,
                None,
                &context.tree,
                context.garden.as_ref(),
                link,
            );
            println!("  {} {}", "-".blue(), value.yellow());
        }
    }
}

/// Print a list of commands
pub(crate) fn print_commands(commands: &model::MultiVariableMap) {
    println!("{}", "commands:".blue());
    for cmd in commands.keys() {
        println!("  {} {}", "-".blue(), cmd.yellow());
    }
}

/// Print groups
pub(crate) fn print_groups(groups: &model::GroupMap) {
    println!("{}", "groups:".blue());
    for group in groups.keys() {
        println!("  {} {}", "-".blue(), group.yellow());
    }
}

/// Print gardens
pub(crate) fn print_gardens(gardens: &model::GardenMap) {
    println!("{}", "gardens:".blue());
    for garden in gardens.keys() {
        println!("  {} {}", "-".blue(), garden.yellow());
    }
}

/// Print a command argument list
pub fn print_command_vec(command: &[&str]) {
    // Shell quote the list of commands.
    let cmd_str = shell_words::join(command);
    println!("{} {}", ":".cyan(), cmd_str.green(),);
}

/// Print a string command argument list
pub fn print_command_string_vec(command: &[String]) {
    let str_vec: Vec<&str> = command.iter().map(String::as_str).collect();
    print_command_vec(&str_vec);
}
