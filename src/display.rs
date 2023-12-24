use crate::{eval, git, model};

// Color is an alias for yansi::Paint.
pub(crate) type Color<T> = yansi::Paint<T>;

pub(crate) fn display_missing_tree(tree: &model::Tree, path: &str, verbose: u8) -> String {
    if verbose > 0 {
        format!(
            "{} {} {} {}",
            Color::black("#").bold(),
            Color::black(tree.get_name()).bold(),
            Color::black(path).bold(),
            Color::black("(skipped)").bold()
        )
    } else {
        format!(
            "{} {} {}",
            Color::black("#").bold(),
            Color::black(tree.get_name()).bold(),
            Color::black("(skipped)").bold()
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
                        Color::cyan("#"),
                        Color::blue(tree.get_name()).bold(),
                        Color::blue("["),
                        Color::cyan(&branch),
                        Color::blue("]"),
                        Color::blue(&path_str)
                    );
                }
            }
        }
        format!(
            "{} {} {}",
            Color::cyan("#"),
            Color::blue(tree.get_name()).bold(),
            Color::blue(path_str)
        )
    } else {
        if tree_branches {
            if let Some(path) = tree.canonical_pathbuf() {
                if let Some(branch) = git::branch(&path) {
                    return format!(
                        "{} {} {}{}{}",
                        Color::cyan("#"),
                        Color::blue(tree.get_name()).bold(),
                        Color::blue("["),
                        Color::cyan(&branch),
                        Color::blue("]")
                    );
                }
            }
        }
        format!(
            "{} {}",
            Color::cyan("#"),
            Color::blue(tree.get_name()).bold()
        )
    }
}

/// Print a tree if it exists, otherwise print a missing tree
pub(crate) fn print_tree(
    tree: &model::Tree,
    tree_branches: bool,
    verbose: u8,
    quiet: bool,
) -> bool {
    if let Ok(path) = tree.path_as_ref() {
        // Sparse gardens/missing trees are expected. Skip these entries.
        if !std::path::PathBuf::from(&path).exists() {
            if !quiet {
                eprintln!("{}", display_missing_tree(tree, path, verbose));
            }
            return false;
        }

        print_tree_details(tree, tree_branches, verbose, quiet);
        return true;
    } else if !quiet {
        eprintln!("{}", display_missing_tree(tree, "[invalid-path]", verbose));
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
    if !quiet {
        if let Ok(path) = tree.path_as_ref() {
            eprintln!("{}", display_tree(tree, path, tree_branches, verbose));
        }
    }
}

/// Print an ungrown / missing tree.
pub(crate) fn print_missing_tree(tree: &model::Tree, path: &str, verbose: u8) {
    if verbose > 0 {
        println!(
            "{} {} {}",
            Color::red("-").dimmed(),
            Color::red(tree.get_name()),
            Color::red(path).dimmed()
        );
    } else {
        println!(
            "{} {}",
            Color::red("-").dimmed(),
            Color::red(tree.get_name())
        );
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
            Color::green("+"),
            Color::green(tree.get_name()).bold(),
            Color::green(path),
            Color::yellow("->").bold(),
            Color::blue(symlink).bold()
        );
    } else {
        println!(
            "{} {} {} {}",
            Color::green("+"),
            Color::green(tree.get_name()).bold(),
            Color::yellow("->").bold(),
            Color::blue(symlink).bold()
        );
    }
}

/// Print the description, url, remotes and links for a tree
pub(crate) fn print_tree_extended_details(
    app_context: &model::ApplicationContext,
    context: &model::TreeContext,
    tree: &model::Tree,
    display_worktrees: bool,
) {
    let config = match context.config {
        Some(config_id) => app_context.get_config(config_id),
        None => app_context.get_root_config(),
    };
    if !tree.description.is_empty() {
        println!("{}", Color::cyan(&tree.description));
    }
    if tree.is_worktree && !display_worktrees {
        return;
    }
    if !tree.remotes.is_empty() {
        println!("{}", Color::blue("remotes:"));
        for (name, remote) in &tree.remotes {
            let value = eval::tree_value(
                app_context,
                config,
                remote.get_expr(),
                &context.tree,
                context.garden.as_ref(),
            );
            println!(
                "  {}{} {}",
                Color::blue(name),
                Color::blue(":"),
                Color::yellow(value)
            );
        }
    }
    if !tree.links.is_empty() {
        println!("{}", Color::blue("links:"));
        for link in &tree.links {
            let value = eval::tree_value(
                app_context,
                config,
                link.get_expr(),
                &context.tree,
                context.garden.as_ref(),
            );
            println!("  {} {}", Color::blue("-"), Color::yellow(value));
        }
    }
}
