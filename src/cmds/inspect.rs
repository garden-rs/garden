use anyhow::Result;
use argparse;

use super::super::cmd;
use super::super::model;
use super::super::model::Color;
use super::super::query;


/// Main entry point for the "garden exec" command
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let options = &mut app.options;
    let config = &mut app.config;

    let mut query: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden inspect - query tree status");

        ap.refer(&mut query)
            .add_argument("query", argparse::List,
                          "gardens/groups/trees to exec (tree queries)");

        options.args.insert(0, "garden exec".to_string());
        cmd::parse_args(ap, options.args.to_vec());
    }
    if query.is_empty() {
        query.push(".".to_string());
    }

    if options.is_debug("inspect") {
        debug!("query: {:?}", query);
    }

    inspect(config, options.verbose, &query)
}


/// Execute a command over every tree in the evaluated tree query.
pub fn inspect(
    config: &mut model::Configuration,
    verbose: bool,
    queries: &Vec<String>,
) -> Result<()> {

    for query in queries {
        // Resolve the tree query into a vector of tree contexts.
        let contexts = query::resolve_trees(config, query);
        // Loop over each context and inspect the tree.
        for context in &contexts {
            let tree = &config.trees[context.tree];
            let path = tree.path.value.as_ref().unwrap();

            // Sparse gardens/missing trees are ok -> skip these entries.
            if !std::path::PathBuf::from(&path).exists() {
                if verbose {
                    println!("{} {}  {}",
                             Color::red("-").dimmed(),
                             Color::red(&tree.name),
                             Color::red(&path).dimmed());
                } else {
                    println!("{} {}",
                             Color::red("-").dimmed(),
                             Color::red(&tree.name));
                }
                continue;
            }

            if tree.is_symlink {
                if verbose {
                    println!("{} {}  {} {} {}",
                             Color::green("+"),
                             Color::green(&tree.name).bold(),
                             Color::green(&path),
                             Color::yellow("->").bold(),
                             Color::blue(&tree.symlink.value.as_ref().unwrap()).bold());
                } else {
                    println!("{} {} {} {}",
                             Color::green("+"),
                             Color::green(&tree.name).bold(),
                             Color::yellow("->").bold(),
                             Color::blue(tree.symlink.value.as_ref().unwrap()).bold());
                }
            } else {
                if verbose {
                    println!("{} {}  {}",
                             Color::green("+"),
                             Color::green(&tree.name).bold(),
                             Color::green(&path));
                } else {
                    println!("{} {}",
                             Color::green("+"), Color::green(&tree.name).bold());
                }
            }
        }
    }

    Ok(())
}
