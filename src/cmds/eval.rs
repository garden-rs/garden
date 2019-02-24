extern crate glob;

use ::eval;
use ::config;
use ::model;
use ::query;


pub fn main(options: &mut model::CommandOptions) {
    options.args.insert(0, "garden eval".to_string());

    let mut expr = String::new();
    let mut tree = String::new();
    let mut garden = String::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden eval - evaluate expressions");

        ap.refer(&mut expr).required()
            .add_argument("garden-expr", argparse::Store,
                          "gardens expression to evaluate");

        ap.refer(&mut tree)
            .add_argument("tree", argparse::Store, "tree to evaluate");

        ap.refer(&mut garden)
            .add_argument("garden", argparse::Store, "garden to evaluate");

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

    if tree.is_empty() {
        println!("{}", eval::value(&mut cfg, &expr));
        return;
    }

    // Evaluate and print the garden expression.
    let mut ctx = model::TreeContext {
        tree: 0,
        garden: None
    };
    if let Some(context) = query::tree_by_name(&cfg, &tree, None) {
        ctx.tree = context.tree;
    } else {
        error!("unable to find '{}': No tree exists with that name", tree);
    }

    if !garden.is_empty() {
        let pattern = glob::Pattern::new(&garden).unwrap();
        let contexts = query::garden_trees(&cfg, &pattern);

        if contexts.is_empty() {
            error!("unable to find '{}': No garden exists with that name",
                   garden);
        }

        let mut found = false;
        for current_ctx in &contexts {
            if current_ctx.tree == ctx.tree {
                ctx.garden = current_ctx.garden;
                found = true;
                break;
            }
        }

        if !found {
            error!("invalid arguments: '{}' is not part of the '{}' garden",
                   tree, garden);
        }
    }

    // Evaluate and print the garden expression.
    let value = eval::tree_value(&mut cfg, &expr, ctx.tree, ctx.garden);
    println!("{}", value);
}
