use anyhow::Result;

use super::super::cmd;
use super::super::eval;
use super::super::model;
use super::super::query;


pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
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

        ap.refer(&mut expr).required().add_argument(
            "expr",
            argparse::Store,
            "garden expression to evaluate",
        );

        ap.refer(&mut tree).add_argument(
            "tree",
            argparse::Store,
            "tree to evaluate",
        );

        ap.refer(&mut garden).add_argument(
            "garden",
            argparse::Store,
            "garden to evaluate",
        );

        options.args.insert(0, "garden eval".to_string());
        cmd::parse_args(ap, options.args.to_vec());
    }

    if tree.is_empty() {
        println!("{}", eval::value(config, &expr));
        return Ok(());
    }

    if !garden.is_empty() {
        garden_opt = Some(garden);
    }

    // Evaluate and print the garden expression.
    let ctx = query::tree_context(config, &tree, garden_opt)?;
    let value = eval::tree_value(config, &expr, ctx.tree, ctx.garden);
    println!("{}", value);

    Ok(())
}
