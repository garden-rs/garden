use anyhow::Result;
use clap::Parser;

use crate::{display, model, query};

/// Query tree status
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct InspectOptions {
    /// Display worktrees
    #[arg(short, long, default_value_t = false)]
    worktrees: bool,
    /// Display details for all trees, including missing trees
    #[arg(short, long, default_value_t = false)]
    all: bool,
    /// Tree query for the gardens, groups or trees to inspect
    queries: Vec<String>,
}

/// Main entry point for the "garden inspect" command
pub fn main(app_context: &model::ApplicationContext, options: &mut InspectOptions) -> Result<()> {
    if options.queries.is_empty() {
        options.queries.push(".".into());
    }
    if app_context.options.debug_level("inspect") > 0 {
        debug!("queries: {:?}", options.queries);
    }
    let verbose = app_context.options.verbose;
    let config = app_context.get_root_config_mut();
    inspect(
        app_context,
        config,
        verbose,
        options.worktrees,
        options.all,
        &options.queries,
    )
}

/// Inspect every tree in the evaluated tree query
fn inspect(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    verbose: u8,
    display_worktrees: bool,
    display_all: bool,
    queries: &[String],
) -> Result<()> {
    for query in queries {
        // Resolve the tree query into a vector of tree contexts.
        let contexts = query::resolve_trees(app_context, config, query);
        // Loop over each context and inspect the tree.
        for context in &contexts {
            let config = match context.config {
                Some(config_id) => app_context.get_config(config_id),
                None => config,
            };
            let tree = match config.trees.get(&context.tree) {
                Some(tree) => tree,
                None => continue,
            };
            let path = tree.path_as_ref()?;
            // Sparse gardens/missing trees are ok -> skip these entries.
            if !std::path::PathBuf::from(path).exists() {
                display::print_missing_tree(tree, path, verbose);
                if display_all {
                    display::print_tree_extended_details(
                        app_context,
                        context,
                        tree,
                        display_worktrees,
                    );
                    println!();
                }
                continue;
            }

            if tree.is_symlink {
                display::print_symlink_tree_entry(tree, path, verbose);
                continue;
            }

            display::print_tree(tree, true, verbose, false);
            display::print_tree_extended_details(app_context, context, tree, display_worktrees);
        }
    }

    Ok(())
}
