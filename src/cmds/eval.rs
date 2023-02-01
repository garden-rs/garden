use anyhow::Result;
use clap::Parser;

use super::super::eval;
use super::super::model;
use super::super::query;

/// Evaluate garden expressions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct EvalOptions {
    /// Expression to evaluate
    expr: String,
    /// Tree within which to evaluate
    tree: Option<String>,
    /// Garden within which to evaluate
    garden: Option<String>,
}

/// Evaluate a garden expression using the Eval parameters
pub fn main(app: &mut model::ApplicationContext, eval: &EvalOptions) -> Result<()> {
    let config = app.get_root_config_mut();
    let mut garden_opt: Option<&str> = None;
    if let Some(garden) = &eval.garden {
        garden_opt = Some(garden.as_str());
    }

    match &eval.tree {
        None => {
            // Evaluate and print the expression in global scope. No trees or gardens
            // were provided so only the top-level variables are included.
            println!("{}", eval::value(config, &eval.expr));
        }
        Some(tree) => {
            // Evaluate and print the garden expression.
            let ctx = query::tree_context(config, tree, garden_opt)?;
            let value = eval::tree_value(config, &eval.expr, &ctx.tree, ctx.garden.as_ref());
            println!("{value}");
        }
    }

    Ok(())
}
