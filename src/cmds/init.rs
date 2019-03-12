use ::cmd;
use ::eval;
use ::model;
use ::query;


/// Main entry point for the "garden exec" command
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(app: &mut model::ApplicationContext) {
    let options = &mut app.options;
    let config = &mut app.config;

    let mut expr = String::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.stop_on_first_argument(true);
        ap.set_description("garden exec - run commands inside gardens");

        ap.refer(&mut expr).required()
            .add_argument("tree-expr", argparse::Store,
                          "gardens/trees to initialize (tree expression)");

        options.args.insert(0, "garden init".to_string());
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let quiet = options.quiet;
    let verbose = options.verbose;
    let exit_status = init(config, quiet, verbose, &expr);
    std::process::exit(exit_status);
}


/// Execute a command over every tree in the evaluated tree expression.
pub fn init(
    config: &mut model::Configuration,
    quiet: bool,
    verbose: bool,
    expr: &str,
) -> i32 {
    let contexts = query::resolve_trees(config, expr);
    let mut exit_status: i32 = 0;

    for ctx in &contexts {
        let name = config.trees[ctx.tree].name.to_string();
        let path = config.trees[ctx.tree].path.value.as_ref().unwrap().to_string();

        if !quiet {
            if verbose {
                eprintln!("# {}: {}", name, path);
            } else {
                eprintln!("# {}", name);
            }
        }

        let pathbuf = std::path::PathBuf::from(&path);
        if !pathbuf.exists() {
            let parent = match pathbuf.parent() {
                Some(result) => result,
                None => {
                    error!("unable to create parent directory for '{}'", path);
                }
            };

            if let Err(err) = std::fs::create_dir_all(&parent) {
                error!("unable to create '{}': {}", path, err);
            }

            if config.trees[ctx.tree].remotes.is_empty() {
                continue;
            }

            // The first remote is "origin" by convention
            let remote = config.trees[ctx.tree].remotes[0].clone();
            let url = eval::tree_value(config, &remote.expr,
                                       ctx.tree, ctx.garden);

            let cmd = ["git", "clone", url.as_ref(), path.as_ref()];
            let exec = cmd::exec_cmd(&cmd);
            let status = cmd::status(exec.join());
            if status != 0 {
                exit_status = status as i32;
            }
        }
    }

    // Return the last non-zero exit status.
    exit_status
}
