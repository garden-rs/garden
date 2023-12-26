use anyhow::Result;
use clap::Parser;

use crate::{display, model, query};

/// Query tree status
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct ListOptions {
    /// Display details for all trees, including missing trees
    #[arg(short, long, default_value_t = false)]
    all: bool,
    /// Do not list commands
    #[arg(long, short = 'c', default_value_t = false)]
    no_commands: bool,
    /// Display worktrees
    #[arg(short, long, default_value_t = false)]
    worktrees: bool,
    /// Tree query for the gardens, groups or trees to display
    queries: Vec<String>,
}

/// Main entry point for the "garden ls" command
pub fn main(app_context: &model::ApplicationContext, options: &mut ListOptions) -> Result<()> {
    if options.queries.is_empty() {
        options.queries.push("@*".into());
    }
    list(app_context, options)
}

/// List tree details
fn list(app_context: &model::ApplicationContext, options: &ListOptions) -> Result<()> {
    let config = app_context.get_root_config();
    let display_all = options.all;
    let display_worktrees = options.worktrees;
    let show_commands = !options.no_commands;
    let verbose = app_context.options.verbose;
    let mut needs_newline = false;

    if app_context.options.debug_level("list") > 0 {
        debug!("queries: {:?}", options.queries);
    }

    for query in &options.queries {
        // Resolve the tree query into a vector of tree contexts.
        let contexts = query::resolve_trees(app_context, config, query);
        // Loop over each context and display the tree.
        for (idx, context) in contexts.iter().enumerate() {
            let config = match context.config {
                Some(config_id) => app_context.get_config(config_id),
                None => config,
            };
            let tree = match config.trees.get(&context.tree) {
                Some(tree) => tree,
                None => continue,
            };
            let path = match tree.path_as_ref() {
                Ok(path) => path,
                Err(_) => continue,
            };
            // Sparse gardens/missing trees are ok -> skip these entries.
            if !std::path::PathBuf::from(path).exists() {
                if needs_newline {
                    println!();
                }
                display::print_missing_tree(tree, path, verbose);
                if display_all {
                    display::print_tree_extended_details(
                        app_context,
                        context,
                        tree,
                        display_worktrees,
                    );
                    if show_commands && !tree.commands.is_empty() {
                        display::print_commands(&tree.commands);
                    }
                }
                needs_newline = display_all;
                continue;
            }

            if tree.is_symlink {
                if needs_newline {
                    println!();
                }
                display::print_symlink_tree_entry(tree, path, verbose);
                needs_newline = false;
                continue;
            }

            if idx > 0 {
                println!();
            }
            display::print_tree(tree, config.tree_branches, verbose, false);
            display::print_tree_extended_details(app_context, context, tree, display_worktrees);
            if show_commands && !tree.commands.is_empty() {
                display::print_commands(&tree.commands);
            }
            needs_newline = true;
        }
    }

    if !config.groups.is_empty() {
        println!();
        display::print_groups(&config.groups);
    }

    if !config.gardens.is_empty() {
        println!();
        display::print_gardens(&config.gardens);
    }

    if show_commands && !config.commands.is_empty() {
        println!();
        display::print_commands(&config.commands);
    }

    Ok(())
}
