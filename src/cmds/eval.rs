extern crate glob;

use ::eval;
use ::model;
use ::query;


pub fn main(app: &mut model::ApplicationContext) -> i32 {
    let config = &mut app.config;
    let options = &mut app.options;

    let mut expr = String::new();
    let mut tree = String::new();
    let mut garden = String::new();
    let mut garden_opt: Option<String> = None;

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden eval - evaluate garden expressions");

        ap.refer(&mut expr).required()
            .add_argument("expr", argparse::Store,
                          "garden expression to evaluate");

        ap.refer(&mut tree)
            .add_argument("tree", argparse::Store, "tree to evaluate");

        ap.refer(&mut garden)
            .add_argument("garden", argparse::Store, "garden to evaluate");

        options.args.insert(0, "garden eval".to_string());
        return_on_err!(ap.parse(options.args.to_vec(),
                                &mut std::io::stdout(),
                                &mut std::io::stderr()));
    }

    if tree.is_empty() {
        println!("{}", eval::value(config, &expr));
        return 0;
    }

    if !garden.is_empty() {
        garden_opt = Some(garden);
    }

    // Evaluate and print the garden expression.
    match query::tree_context(config, &tree, garden_opt) {
        Ok(ctx) => {
            let value = eval::tree_value(config, &expr,
                                         ctx.tree, ctx.garden);
            println!("{}", value);
            0
        }
        Err(err) => {
            error!("{}", err);
        }
    }
}
