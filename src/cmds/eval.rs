use anyhow::Result;
use clap::Parser;

use crate::{eval, model, query};

/// Evaluate garden expressions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct EvalOptions {
    /// Set variables using 'name=value' expressions
    #[arg(long, short = 'D')]
    define: Vec<String>,
    /// Expression to evaluate
    expr: String,
    /// Tree within which to evaluate
    tree: Option<String>,
    /// Garden within which to evaluate
    garden: Option<String>,
}

/// Evaluate a garden expression using the Eval parameters
pub fn main(app_context: &model::ApplicationContext, eval: &EvalOptions) -> Result<()> {
    app_context
        .get_root_config_mut()
        .apply_defines(&eval.define);
    match eval.tree.as_ref() {
        None => {
            // Evaluate and print the expression in global scope. No trees or gardens
            // were provided so only the top-level variables are included.
            let config = app_context.get_root_config();
            let value = eval::value(app_context, config, &eval.expr);
            println!("{value}");
        }
        Some(tree) => {
            // Evaluate and print the garden expression.
            let garden = eval.garden.as_deref();
            let ctx = query::find_tree(app_context, app_context.get_root_id(), tree, garden)?;
            let graft_config = ctx.config.map(|graft_id| app_context.get_config(graft_id));
            let value = eval::tree_value(
                app_context,
                app_context.get_root_config(),
                graft_config,
                &eval.expr,
                &ctx.tree,
                ctx.garden.as_ref(),
            );
            println!("{value}");
        }
    }

    Ok(())
}
