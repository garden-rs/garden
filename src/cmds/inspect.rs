use anyhow::Result;

use super::super::cmd;
use super::super::model;
use super::super::model::Color;
use super::super::query;

/// Main entry point for the "garden exec" command
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let mut query: Vec<String> = Vec::new();
    parse_args(&mut app.options, &mut query);

    let verbose = app.options.verbose;
    let config = app.get_root_config_mut();
    inspect(config, verbose, &query)
}

/// Parse "inspect" arguments.
fn parse_args(options: &mut model::CommandOptions, query: &mut Vec<String>) {
    let mut ap = argparse::ArgumentParser::new();
    ap.set_description("garden inspect - query tree status");

    ap.refer(query).add_argument(
        "query",
        argparse::List,
        "gardens/groups/trees to exec (tree queries)",
    );

    options.args.insert(0, "garden inspect".into());
    cmd::parse_args(ap, options.args.to_vec());

    if query.is_empty() {
        query.push(".".into());
    }

    if options.is_debug("inspect") {
        debug!("query: {:?}", query);
    }
}

/// Execute a command over every tree in the evaluated tree query.
pub fn inspect(config: &mut model::Configuration, verbose: u8, queries: &[String]) -> Result<()> {
    for query in queries {
        // Resolve the tree query into a vector of tree contexts.
        let contexts = query::resolve_trees(config, query);
        // Loop over each context and inspect the tree.
        for context in &contexts {
            let tree = &config.trees[context.tree];
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
