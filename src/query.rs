use super::errors;
use super::eval;
use super::model;
use super::path;
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
    let pattern = &tree_query.pattern;

    if tree_query.include_gardens {
        result = garden_trees(config, pattern);
        if !result.is_empty() {
            return result;
        }
    }

    if tree_query.include_groups {
        for (name, group) in &config.groups {
            // Find the matching group
            if !pattern.matches(name) {
                continue;
            }
            // Matching group found, collect its trees
            result.append(&mut trees_from_group(config, None, group));
        }
        if !result.is_empty() {
            return result;
        }
    }

    // No matching gardens or groups were found.
    // Search for matching trees.
    if tree_query.include_trees {
        result.append(&mut trees(config, pattern));
        if !result.is_empty() {
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

    for (name, garden) in &config.gardens {
        if !pattern.matches(name) {
            continue;
        }
        result.append(&mut trees_from_garden(config, garden));
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
        let pattern = match glob::Pattern::new(group) {
            Ok(value) => value,
            Err(_) => continue,
        };
        // Loop over configured groups to find the matching groups
        for (name, cfg_group) in &config.groups {
            if !pattern.matches(name) {
                continue;
            }
            // Match found -- take all of the discovered trees.
            result.append(&mut trees_from_group(
                config,
                Some(garden.get_name()),
                cfg_group,
            ));
        }
    }

    // Collect indexes for each tree in this garden
    for tree in &garden.trees {
        result.append(&mut trees_from_pattern(
            config,
            tree,
            Some(garden.get_name()),
            None,
        ));
    }

    result
}

/// Return the tree contexts for a garden
pub fn trees_from_group(
    config: &model::Configuration,
    garden: Option<&model::GardenName>,
    group: &model::Group,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();

    // Collect indexes for each tree in this group
    for tree in &group.members {
        result.append(&mut trees_from_pattern(
            config,
            tree,
            garden,
            Some(group.get_name()),
        ));
    }

    result
}

/// Find a tree by name
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - tree: Tree name `&str`
/// - garden_idx: `Option<garden::model::GardenName>`

pub fn tree_from_name(
    config: &model::Configuration,
    tree_name: &str,
    garden_name: Option<&model::GardenName>,
    group: Option<&model::GardenName>,
) -> Option<model::TreeContext> {
    // Collect tree indexes for the configured trees
    if let Some(tree) = config.trees.get(tree_name) {
        return Some(model::TreeContext::new(
            tree.get_name(),
            config.get_id(),
            garden_name.cloned(),
            group.cloned(),
        ));
    }

    // Try to find the specified name on the filesystem if no tree was found
    // that matched the specified name.  Matching trees are found by matching
    // tree paths against the specified name.
    if let Some(ctx) = tree_from_path(config, tree_name) {
        return Some(ctx);
    }

    None
}

/// Find trees matching a pattern
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - tree: Tree name pattern `&str`
/// - garden_name: `Option<garden::model::GardenName>`

pub fn trees_from_pattern(
    config: &model::Configuration,
    tree: &str,
    garden_name: Option<&model::GardenName>,
    group: Option<&model::GroupName>,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();
    let pattern = match glob::Pattern::new(tree) {
        Ok(value) => value,
        Err(_) => return result,
    };

    // Collect tree indexes for the configured trees
    for (tree_name, cfg_tree) in &config.trees {
        if pattern.matches(tree_name) {
            // Tree found
            result.push(model::TreeContext::new(
                cfg_tree.get_name(),
                config.get_id(),
                garden_name.cloned(),
                group.cloned(),
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

/// Return a tree context for the specified path string.
pub fn tree_from_path(config: &model::Configuration, path: &str) -> Option<model::TreeContext> {
    tree_from_pathbuf(config, &std::path::PathBuf::from(path))
}

/// Return a tree context for the specified path.
pub fn tree_from_pathbuf(
    config: &model::Configuration,
    path: &std::path::Path,
) -> Option<model::TreeContext> {
    let pathbuf = match path.canonicalize() {
        Ok(canon) => canon,
        Err(_) => return None,
    };

    for (name, tree) in &config.trees {
        let tree_path = match tree.path_as_ref() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let tree_canon = match std::path::PathBuf::from(tree_path).canonicalize() {
            Ok(value) => value,
            Err(_) => continue,
        };
        if pathbuf == tree_canon {
            return Some(model::TreeContext::new(name, config.get_id(), None, None));
        }
    }

    None
}

/// Return the name of an existing tree from the specified path.

pub fn tree_name_from_path(
    config: &model::Configuration,
    path: &std::path::Path,
) -> Option<String> {
    tree_name_from_abspath(config, &path::abspath(path))
}

/// Return the name of an existing tree from an absolute path.

pub fn tree_name_from_abspath(
    config: &model::Configuration,
    path: &std::path::Path,
) -> Option<String> {
    // Do we already have a tree with this tree path?
    for tree in config.trees.values() {
        // Skip entries that do not exist on disk.
        if !tree.path_is_valid() {
            continue;
        }
        let tree_path_str = match tree.path_as_ref() {
            Ok(path_str) => path_str,
            Err(_) => continue,
        };
        // Check if this tree matches the specified path.
        let tree_pathbuf = std::path::PathBuf::from(tree_path_str);
        if let Ok(canon_path) = tree_pathbuf.canonicalize() {
            if canon_path == path {
                // Existing tree found: use the configured name.
                return Some(tree.get_name().to_string());
            }
        }
    }

    None
}

/// Returns tree contexts matching the specified pattern

fn trees(config: &model::Configuration, pattern: &glob::Pattern) -> Vec<model::TreeContext> {
    let mut result = Vec::new();
    for (tree_name, tree) in &config.trees {
        if pattern.matches(tree_name) {
            result.push(model::TreeContext::new(
                tree.get_name(),
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
) -> Result<model::TreeContext, errors::GardenError> {
    let mut ctx = model::TreeContext::new("", config.get_id(), None, None);
    // TODO: grafted trees
    if let Some(context) = tree_from_name(config, tree, None, None) {
        ctx.tree = context.tree;
    } else {
        return Err(errors::GardenError::TreeNotFound { tree: tree.into() });
    }

    if let Some(garden_name) = garden {
        let pattern = glob::Pattern::new(garden_name).map_err(|_| {
            errors::GardenError::GardenPatternError {
                garden: garden_name.into(),
            }
        })?;
        let contexts = query::garden_trees(config, &pattern);

        if contexts.is_empty() {
            return Err(errors::GardenError::GardenNotFound {
                garden: garden_name.into(),
            });
        }

        let mut found = false;
        for current_ctx in &contexts {
            if current_ctx.tree == ctx.tree {
                ctx.garden = current_ctx.garden.clone();
                found = true;
                break;
            }
        }

        if !found {
            return Err(errors::GardenError::InvalidGardenArgument {
                tree: tree.into(),
                garden: garden_name.into(),
            });
        }
    }

    Ok(ctx)
}

pub fn find_tree(
    app: &model::ApplicationContext,
    id: model::ConfigId,
    tree: &str,
    garden: Option<&str>,
) -> Result<model::TreeContext, errors::GardenError> {
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

/// Return a path that that is either the tree's path or the tree's shared worktree path.
pub fn shared_worktree_path(config: &model::Configuration, ctx: &model::TreeContext) -> String {
    let tree = match config.trees.get(&ctx.tree) {
        Some(tree) => tree,
        None => return String::new(),
    };
    if tree.is_worktree {
        let worktree = eval::tree_value(
            config,
            tree.worktree.get_expr(),
            &ctx.tree,
            ctx.garden.as_ref(),
        );
        if let Some(parent_ctx) =
            query::tree_from_name(config, &worktree, ctx.garden.as_ref(), ctx.group.as_ref())
        {
            if let Some(path) = config
                .trees
                .get(&parent_ctx.tree)
                .and_then(|tree| tree.path_as_ref().ok())
            {
                return path.to_string();
            }
        }
    }

    if let Ok(path) = tree.path_as_ref() {
        return path.to_string();
    }

    tree.get_name().to_string()
}
