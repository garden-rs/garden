use super::errors::GardenError;
use super::model;
use super::query;
use super::syntax;


/// Resolve a tree query into a `Vec<garden::model::TreeContext>`.
///
/// Parameters:
/// - `config`: `&garden::model::Configuration`.
/// - `query`: Tree query `&str`.
/// Returns:
/// - `Vec<garden::model::TreeContext>`

pub fn resolve_trees(config: &model::Configuration, query: &str) -> Vec<model::TreeContext> {
    let mut result = Vec::new();
    let tree_query = model::TreeQuery::new(query);
    let ref pattern = tree_query.pattern;

    if tree_query.include_gardens {
        result = garden_trees(config, pattern);
        if result.len() > 0 {
            return result;
        }
    }

    if tree_query.include_groups {
        for group in &config.groups {
            // Find the matching group
            if !pattern.matches(group.get_name()) {
                continue;
            }
            // Matching group found, collect its trees
            result.append(&mut trees_from_group(config, None, group));
        }
        if result.len() > 0 {
            return result;
        }
    }

    // No matching gardens or groups were found.
    // Search for matching trees.
    if tree_query.include_trees {
        result.append(&mut trees(config, pattern));
        if result.len() > 0 {
            return result;
        }
    }

    // Lowest precedence: match paths on the filesystem.
    // The pattern is a default non-special pattern, and its value points to an
    // existing tree on the filesystem.  Look up the tree context for this
    // entry and use the matching tree.
    if tree_query.is_default {
        if let Some(ctx) = tree_from_path(config, &tree_query.query) {
            result.push(ctx);
        }
    }

    result
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
        if !pattern.matches(garden.get_name()) {
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
        // Create a glob pattern for the group entry
        let pattern = match glob::Pattern::new(&group) {
            Ok(value) => value,
            Err(_) => continue,
        };
        // Loop over configured groups to find the matching groups
        for cfg_group in &config.groups {
            if !pattern.matches(cfg_group.get_name()) {
                continue;
            }
            // Match found -- take all of the discovered trees.
            result.append(&mut trees_from_group(
                config,
                Some(garden.get_index()),
                cfg_group,
            ));
        }
    }

    // Collect indexes for each tree in this garden
    for tree in &garden.trees {
        result.append(&mut trees_from_pattern(
            config,
            tree,
            Some(garden.get_index()),
            None,
        ));
    }

    result
}

/// Return the tree contexts for a garden
pub fn trees_from_group(
    config: &model::Configuration,
    garden: Option<model::GardenIndex>,
    group: &model::Group,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();

    // Collect indexes for each tree in this group
    for tree in &group.members {
        result.append(&mut trees_from_pattern(
            config,
            tree,
            garden,
            Some(group.get_index()),
        ));
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
    group_idx: Option<model::GroupIndex>,
) -> Option<model::TreeContext> {


    // Collect tree indexes for the configured trees
    for (tree_idx, cfg_tree) in config.trees.iter().enumerate() {
        if cfg_tree.get_name() == tree {
            // Tree found
            return Some(model::TreeContext::new(
                tree_idx,
                config.get_id(),
                garden_idx,
                group_idx,
            ));
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

/// Find trees matching a pattern
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - tree: Tree name pattern `&str`
/// - garden_idx: `Option<garden::model::GardenIndex>`

pub fn trees_from_pattern(
    config: &model::Configuration,
    tree: &str,
    garden_idx: Option<model::GardenIndex>,
    group_idx: Option<model::GroupIndex>,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();
    let pattern = match glob::Pattern::new(tree) {
        Ok(value) => value,
        Err(_) => return result,
    };

    // Collect tree indexes for the configured trees
    for (tree_idx, cfg_tree) in config.trees.iter().enumerate() {
        if pattern.matches(cfg_tree.get_name()) {
            // Tree found
            result.push(model::TreeContext::new(
                tree_idx,
                config.get_id(),
                garden_idx,
                group_idx,
            ));
        }
    }

    // Try to find the specified name on the filesystem if no tree was found
    // that matched the specified name.  Matching trees are found by matching
    // tree paths against the specified name.
    if result.is_empty() {
        if let Some(ctx) = tree_from_path(config, tree) {
            result.push(ctx);
        }
    }


    result
}


/// Return a tree context for the specified filesystem path.

pub fn tree_from_path(config: &model::Configuration, path: &str) -> Option<model::TreeContext> {
    let pathbuf = match std::path::PathBuf::from(path).canonicalize() {
        Ok(canon) => canon.to_path_buf(),
        Err(_) => return None,
    };

    for (idx, tree) in config.trees.iter().enumerate() {
        let tree_path = match tree.path_as_ref() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let tree_canon = match std::path::PathBuf::from(tree_path).canonicalize() {
            Ok(value) => value,
            Err(_) => continue,
        };
        if pathbuf == tree_canon {
            return Some(model::TreeContext::new(
                idx as model::TreeIndex,
                config.get_id(),
                None,
                None,
            ));
        }
    }

    None
}

/// Returns tree contexts matching the specified pattern

fn trees(config: &model::Configuration, pattern: &glob::Pattern) -> Vec<model::TreeContext> {

    let mut result = Vec::new();
    for (tree_idx, tree) in config.trees.iter().enumerate() {
        if pattern.matches(tree.get_name()) {
            result.push(model::TreeContext::new(
                tree_idx,
                config.get_id(),
                None,
                None,
            ));
        }
    }

    result
}


/// Return a Result<garden::model::TreeContext, garden::errors::GardenError>
/// when the tree and optional garden are present.

pub fn tree_context(
    config: &model::Configuration,
    tree: &str,
    garden: Option<&str>,
) -> Result<model::TreeContext, GardenError> {

    let mut ctx = model::TreeContext::new(0, config.get_id(), None, None);
    // TODO: grafted trees
    if let Some(context) = tree_from_name(&config, tree, None, None) {
        ctx.tree = context.tree;
    } else {
        return Err(GardenError::TreeNotFound { tree: tree.into() }.into());
    }

    if let Some(garden_name) = garden {
        let pattern = glob::Pattern::new(garden_name).map_err(|_| {
            GardenError::GardenPatternError { garden: garden_name.into() }
        })?;
        let contexts = query::garden_trees(config, &pattern);

        if contexts.is_empty() {
            return Err(
                GardenError::GardenNotFound { garden: garden_name.into() }.into(),
            );
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
            return Err(
                GardenError::InvalidGardenArgument {
                    tree: tree.into(),
                    garden: garden_name.into(),
                }.into(),
            );
        }
    }

    Ok(ctx)
}


pub fn find_tree(
    app: &model::ApplicationContext,
    id: model::ConfigId,
    tree: &str,
    garden: Option<&str>,
) -> Result<model::TreeContext, GardenError> {

    {
        let config = app.get_config(id);
        if let Some(graft_name) = syntax::graft_basename(tree) {
            if syntax::is_graft(tree) && config.contains_graft(&graft_name) {
                let graft = config.get_graft(&graft_name)?;
                let graft_config = app.get_config(graft.get_id().unwrap());

                if let Some(next_graft) = syntax::trim_graft(tree) {
                    return tree_context(graft_config, &next_graft, garden);
                }
            }
        }
    }

    let config = app.get_config(id);
    tree_context(config, tree, garden)
}
