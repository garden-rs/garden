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

pub fn main<S: Into<String>>(
    config: &mut model::Configuration, expr: S, command: &Vec<String>) {

    let contexts = query::resolve_trees(config, expr);
}
