use anyhow::Result;
use shlex;

use super::super::cmd;
use super::super::errors;
use super::super::eval;
use super::super::model;
use super::super::query;


pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let options = &mut app.options;
    let mut query = String::new();
    let mut tree = String::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden shell - open a shell in a garden environment");

        ap.refer(&mut query).required().add_argument(
            "query",
            argparse::Store,
            "query for trees to build an environment",
        );

        ap.refer(&mut tree).add_argument(
            "tree",
            argparse::Store,
            "tree to chdir into",
        );

        options.args.insert(0, "garden shell".into());
        cmd::parse_args(ap, options.args.to_vec());
    }

    let config = app.get_mut_config();
    let contexts = query::resolve_trees(config, &query);
    if contexts.is_empty() {
        // TODO errors::GardenError::TreeQueryMatchedNoTrees { query: query.into() }
        error!("tree query matched zero trees: '{}'", query);
    }

    let mut context = contexts[0].clone();

    // If a tree's name in the returned contexts exactly matches the tree
    // query that was used to find it then chdir into that tree.
    // This makes it convenient to have gardens and trees with the same name.
    for ctx in &contexts {
        if config.trees[ctx.tree].name == query {
            context.tree = ctx.tree;
            context.garden = ctx.garden;
            context.group = ctx.group;
            break;
        }
    }

    if !tree.is_empty() {
        let mut found = false;

        if let Some(ctx) = query::tree_from_name(config, &tree, None, None) {
            for query_ctx in &contexts {
                if ctx.tree == query_ctx.tree {
                    context.tree = query_ctx.tree;
                    context.garden = query_ctx.garden;
                    context.group = query_ctx.group;
                    found = true;
                    break;
                }
            }
        } else {
            error!("unable to find '{}': No tree exists with that name", tree);
        }
        if !found {
            error!("'{}' was not found in the tree query '{}'", tree, query);
        }
    }

    // Evaluate garden.shell
    let shell_expr = config.shell.clone();
    let shell = eval::tree_value(config, &shell_expr, context.tree, context.garden);

    if let Some(value) = shlex::split(&shell) {
        cmd::exec_in_context(
            config,
            &context,
            /*quiet*/
            true,
            /*verbose*/
            false,
            &value,
        ).map_err(|err| err.into())
    } else {
        Err(
            errors::GardenError::InvalidConfiguration {
                msg: format!("unable to shlex::split '{}'", shell),
            }.into(),
        )
    }
}
