use anyhow::Result;
use clap::Parser;

use super::super::model;
use super::super::model::Color;
use super::super::query;

/// Query tree status
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct InspectOptions {
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
    inspect(app_context, config, verbose, &options.queries)
}

/// Inspect every tree in the evaluated tree query
fn inspect(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    verbose: u8,
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
            if !std::path::PathBuf::from(&path).exists() {
                if verbose > 0 {
                    println!(
                        "{} {}  {}",
                        Color::red("-").dimmed(),
                        Color::red(tree.get_name()),
                        Color::red(&path).dimmed()
                    );
                } else {
                    println!(
                        "{} {}",
                        Color::red("-").dimmed(),
                        Color::red(tree.get_name())
                    );
                }
                continue;
            }

            if tree.is_symlink {
                if verbose > 0 {
                    println!(
                        "{} {}  {} {} {}",
                        Color::green("+"),
                        Color::green(tree.get_name()).bold(),
                        Color::green(&path),
                        Color::yellow("->").bold(),
                        Color::blue(&tree.symlink_as_ref()?).bold()
                    );
                } else {
                    println!(
                        "{} {} {} {}",
                        Color::green("+"),
                        Color::green(tree.get_name()).bold(),
                        Color::yellow("->").bold(),
                        Color::blue(tree.symlink_as_ref()?).bold()
                    );
                }
            } else if verbose > 0 {
                println!(
                    "{} {}  {}",
                    Color::green("+"),
                    Color::green(tree.get_name()).bold(),
                    Color::green(&path)
                );
            } else {
                println!(
                    "{} {}",
                    Color::green("+"),
                    Color::green(tree.get_name()).bold()
                );
            }
        }
    }

    Ok(())
}
