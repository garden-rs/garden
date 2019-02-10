extern crate glob;

use super::model;
use super::query;

/// Resolve garden and tree names into a set of trees
/// Strategy: resolve the trees down to a set of tree indexes paired with an
/// an optional garden context.
///
/// If the names resolve to gardens, each garden is processed independently.
/// Trees that exist in multiple matching gardens will be processed multiple
/// times.
///
/// If the names resolve to trees, each tree is processed independently
/// with no garden context.

pub fn main(config: &mut model::Configuration,
            expr: &String, command: &Vec<String>) {

    let contexts = query::resolve_trees(config, expr);
    debug!("contexts: {:?}", contexts);
    debug!("command: {:?}", command);
}
