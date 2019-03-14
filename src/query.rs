use ::model;
use ::query;


/// Resolve a tree expression into a `Vec<garden::model::TreeContext>`.
///
/// Parameters:
/// - `config`: `&garden::model::Configuration`.
/// - `expr`: Tree expression `&str`.
/// Returns:
/// - `Vec<garden::model::TreeContext>`

pub fn resolve_trees(config: &model::Configuration, expr: &str)
-> Vec<model::TreeContext> {
    let tree_expr = model::TreeExpression::new(expr);
    let ref pattern = tree_expr.pattern;

    // Highest precedence: the pattern is a default non-special
    // pattern, and its value points to an existing tree on the
    // filesystem.  Look up the tree context for this entry and
    // use the matching tree.
    if tree_expr.is_default {
        if let Some(ctx) = tree_from_path(config, &tree_expr.expr) {
            return vec!(ctx);
        }
    }

    if tree_expr.include_gardens {
        let result = garden_trees(config, pattern);
        if result.len() > 0 {
            return result;
        }
    }

    if tree_expr.include_groups {
        let mut result = Vec::new();
        for (idx, group) in config.groups.iter().enumerate() {
            // Find the matching group
            if !pattern.matches(group.name.as_ref()) {
                continue;
            }
            // Matching group found, collect its trees
            for tree in &group.members {
                if let Some(mut tree_ctx) = tree_from_name(config, tree, None) {
                    tree_ctx.group = Some(idx as model::GroupIndex);
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

pub fn garden_trees(
    config: &model::Configuration,
    pattern: &glob::Pattern,
) -> Vec<model::TreeContext> {

    let mut result = Vec::new();

    for garden in &config.gardens {
        if !pattern.matches(garden.name.as_ref()) {
            continue;
        }
        result.append(&mut trees_from_garden(config, &garden));
    }

    result
}


/// Return the tree contexts for a garden
pub fn trees_from_garden(
    config: &model::Configuration,
    garden: &model::Garden,
) -> Vec<model::TreeContext> {

    let mut result = Vec::new();

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
                if let Some(tree_ctx) = tree_from_name(
                        config, tree, Some(garden.index)) {
                    result.push(tree_ctx);
                }
            }
        }
    }

    // Collect indexes for each tree in this garden
    for tree in &garden.trees {
        if let Some(tree_ctx) = tree_from_name(
                config, tree, Some(garden.index)) {
            result.push(tree_ctx);
        }
    }

    result
}


/// Find a tree by name
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - tree: Tree name `&str`
/// - garden_idx: `Option<garden::model::GardenIndex>`

pub fn tree_from_name(
    config: &model::Configuration,
    tree: &str,
    garden_idx: Option<model::GardenIndex>,
) -> Option<model::TreeContext> {


    // Collect tree indexes for the configured trees
    for (tree_idx, cfg_tree) in config.trees.iter().enumerate() {
        if *tree == cfg_tree.name {
            // Tree found
            return Some(model::TreeContext {
                tree: tree_idx,
                garden: garden_idx,
                group: None,
            });
        }
    }

    // Try to find the specified name on the filesystem if no tree was found
    // that matched the specified name.  Matching trees are found by matching
    // tree paths against the specified name.

    if let Some(ctx) = tree_from_path(config, tree) {
        return Some(ctx);
    }

    None
}


/// Return a tree context for the specified filesystem path.

pub fn tree_from_path(
    config: &model::Configuration,
    path: &str,
) -> Option<model::TreeContext> {

    let canon = std::path::PathBuf::from(path).canonicalize();
    if canon.is_err() {
        return None;
    }

    let pathbuf = canon.unwrap().to_path_buf();

    for (idx, tree) in config.trees.iter().enumerate() {
        let tree_path = tree.path.value.as_ref().unwrap();
        let tree_canon = std::path::PathBuf::from(tree_path).canonicalize();
        if tree_canon.is_err() {
            continue;
        }
        if pathbuf == tree_canon.unwrap() {
            return Some(
                model::TreeContext {
                    tree: idx as model::TreeIndex,
                    garden: None,
                    group: None,
                }
            );
        }
    }

    None
}

/// Returns tree contexts matching the specified pattern

fn trees(config: &model::Configuration, pattern: &glob::Pattern)
    -> Vec<model::TreeContext> {

    let mut result = Vec::new();
    for (tree_idx, tree) in config.trees.iter().enumerate() {
        if pattern.matches(tree.name.as_ref()) {
            result.push(
                model::TreeContext {
                    tree: tree_idx,
                    garden: None,
                    group: None,
                }
            );
        }
    }

    result
}


/// Return a Result<garden::model::TreeContext, String> when the tree and
/// optional garden are present.  Err is a String.

pub fn tree_context(config: &model::Configuration,
                    tree: &str, garden: Option<String>)
-> Result<model::TreeContext, String> {
    // Evaluate and print the garden expression.
    let mut ctx = model::TreeContext {
        tree: 0,
        garden: None,
        group: None,
    };
    if let Some(context) = tree_from_name(&config, tree, None) {
        ctx.tree = context.tree;
    } else {
        return Err(format!(
                "unable to find '{}': No tree exists with that name", tree));
    }

    if garden.is_some() {
        let pattern = glob::Pattern::new(garden.as_ref().unwrap()).unwrap();
        let contexts = query::garden_trees(config, &pattern);

        if contexts.is_empty() {
            return Err(format!(
                "unable to find '{}': No garden exists with that name",
                garden.unwrap()));
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
            return Err(format!(
                "invalid arguments: '{}' is not part of the '{}' garden",
                tree, garden.unwrap()));
        }
    }

    Ok(ctx)
}
