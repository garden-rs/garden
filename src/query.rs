use super::model;


/// Resolve a tree expression into a `Vec<garden::model::TreeContext>`.
///
/// Parameters:
/// - `config`: `&garden::model::Configuration`.
/// - `expr`: Tree expression `&String`.
/// Returns:
/// - `Vec<garden::model::TreeContext>`

pub fn resolve_trees(config: &model::Configuration, expr: &String)
    -> Vec<model::TreeContext>
{
    let tree_expr = model::TreeExpression::new(expr);
    let ref pattern = tree_expr.pattern;

    if tree_expr.include_gardens {
        let result = garden_trees(config, pattern);
        if result.len() > 0 {
            return result;
        }
    }

    if tree_expr.include_groups {
        let mut result = Vec::new();
        for group in &config.groups {
            // Find the matching group
            if !pattern.matches(group.name.as_ref()) {
                continue;
            }
            // Matching group found, collect its trees
            for tree in &group.members {
                if let Some(tree_ctx) = tree_by_name(config, tree, None) {
                    result.push(tree_ctx);
                }
            }
        }
        if result.len() > 0 {
            return result;
        }
    }

    // No matching gardens or groups were found.
    // Search for matching trees.
    if tree_expr.include_trees {
        return trees(config, pattern);
    }

    return Vec::new();
}


/// Return tree contexts for every garden matching the specified pattern.
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - pattern: `&glob::Pattern`
fn garden_trees(config: &model::Configuration, pattern: &glob::Pattern)
    -> Vec<model::TreeContext> {

    let mut result = Vec::new();
    let mut garden_idx: model::GardenIndex = 0;

    for garden in &config.gardens {
        if !pattern.matches(garden.name.as_ref()) {
            garden_idx += 1;
            continue;
        }
        // Loop over the garden's groups.
        for group in &garden.groups {
            // Loop over configured groups to find the matching name
            for cfg_group in &config.groups {
                if &cfg_group.name != group {
                    continue;
                }
                // Match found -- loop over each tree in the group and
                // find its index in the configuration.
                for tree in &cfg_group.members {
                    if let Some(tree_ctx) = tree_by_name(
                            config, tree, Some(garden_idx)) {
                        result.push(tree_ctx);
                    }
                }
            }
        }

        // Collect indexes for each tree in this garden
        for tree in &garden.trees {
            if let Some(tree_ctx) = tree_by_name(
                    config, tree, Some(garden_idx)) {
                result.push(tree_ctx);
            }
        }

        // Advance to the next garden index
        garden_idx += 1;
    }

    return result;
}


/// Find a tree by name
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - tree: Tree name `&String`
/// - garden_idx: `Option<garden::model::GardenIndex>`

fn tree_by_name(config: &model::Configuration, tree: &String,
                garden_idx: Option<model::GardenIndex>)
    -> Option<model::TreeContext> {

    let mut tree_idx: model::TreeIndex = 0;
    // Collect tree indexes for the configured trees
    for cfg_tree in &config.trees {
        if *tree == cfg_tree.name {
            // Tree found
            return Some(model::TreeContext {
                tree: tree_idx,
                garden: garden_idx,
            });
        }
        // Advance to the next tree index
        tree_idx += 1;
    }
    return None;
}

/// Returns tree contexts matching the specified pattern

fn trees(config: &model::Configuration, pattern: &glob::Pattern)
    -> Vec<model::TreeContext> {

    let mut result = Vec::new();
    let mut tree_idx: model::TreeIndex = 0;
    for tree in &config.trees {
        if pattern.matches(tree.name.as_ref()) {
            result.push(
                model::TreeContext {
                    tree: tree_idx,
                    garden: None,
                }
            );
        }
        tree_idx += 1;
    }

    return result;
}
