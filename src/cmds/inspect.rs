use ::model;
use ::query;


/// Main entry point for the "garden exec" command
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(app: &mut model::ApplicationContext) {
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
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }
    if query.is_empty() {
        query.push(".".to_string());
    }

    if options.is_debug("inspect") {
        debug!("query: {:?}", query);
    }

    let verbose = options.verbose;
    let exit_status = inspect(config, verbose, &query);
    std::process::exit(exit_status);
}


/// Execute a command over every tree in the evaluated tree query.
pub fn inspect(
    config: &mut model::Configuration,
    verbose: bool,
    queries: &Vec<String>,
) -> i32 {

    // The last error is returned through the exit status.
    let mut exit_status: i32 = 0;

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
                    println!("- {} ({})", tree.name, path);
                } else {
                    println!("- {}", tree.name);
                }
                // Missing trees trigger a non-zero status.
                exit_status = 1;
                continue;
            }

            if tree.is_symlink {
                if verbose {
                    println!("+ {} ({}) -> {}", tree.name, path,
                             tree.symlink.value.as_ref().unwrap());
                } else {
                    println!("+ {} -> {}", tree.name,
                             tree.symlink.value.as_ref().unwrap());
                }
            } else {
                if verbose {
                    println!("+ {} ({})", tree.name, path);
                } else {
                    println!("+ {}", tree.name);
                }
            }
        }
    }

    exit_status
}
