use ::cmd;
use ::eval;
use ::config;
use ::model;
use ::query;


pub fn main(options: &mut model::CommandOptions) {
    options.args.insert(0, "garden shell".to_string());

    let mut expr = String::new();
    let mut tree = String::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description(
            "garden shell - open a shell in a garden environment");

        ap.refer(&mut expr).required()
            .add_argument("expr", argparse::Store, "tree expression evaluate");

        ap.refer(&mut tree)
            .add_argument("tree", argparse::Store, "tree to chdir into");

        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let verbose = options.is_debug("config::new");
    let mut cfg = config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", cfg);
    }

    let contexts = query::resolve_trees(&cfg, &expr);
    if contexts.is_empty() {
        error!("tree expression matched zero trees: '{}'", expr);
    }

    let mut context = contexts[0].clone();

    if !tree.is_empty() {
        let mut found = false;

        if let Some(ctx) = query::tree_by_name(&cfg, &tree, None) {
            for expr_ctx in &contexts {
                if ctx.tree == expr_ctx.tree {
                    context.tree = expr_ctx.tree;
                    context.garden = expr_ctx.garden;
                    found = true;
                    break;
                }
            }
        } else {
            error!("unable to find '{}': No tree exists with that name", tree);
        }
        if !found {
            error!("'{}' was not found in the tree expression '{}'",
                   tree, expr);
        }
    }

    // Evaluate garden.shell
    let shell_expr = cfg.shell.to_string();
    let shell = eval::tree_value(&mut cfg, &shell_expr,
                                 context.tree, context.garden);
    // TODO shlex.split()
    let commands = vec!(shell);
    let exit_status = cmd::exec_in_context(
        &mut cfg, &context, options.quiet, options.verbose, &commands);

    std::process::exit(exit_status);
}
