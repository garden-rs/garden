use crate::{constants, errors, eval, model, path, query, syntax};

/// Resolve a tree query into a `Vec<garden::model::TreeContext>`.
///
/// Parameters:
/// - `config`: `&garden::model::Configuration`.
/// - `query`: Tree query `&str`.
/// Returns:
/// - `Vec<garden::model::TreeContext>`

pub fn resolve_trees(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    query: &str,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();
    let tree_query = model::TreeQuery::new(query);
    let pattern = &tree_query.pattern;

    if tree_query.include_gardens {
        result = garden_trees(app_context, config, graft_config, pattern);
        if !result.is_empty() {
            return result;
        }
    }

    let mut group_found = false;
    if tree_query.include_groups {
        if let Some(graft_cfg) = graft_config {
            for (name, group) in &graft_cfg.groups {
                // Find the matching group
                if !pattern.matches(name) {
                    continue;
                }
                // Matching group found, collect its trees
                result.append(&mut trees_from_group(
                    app_context,
                    config,
                    graft_config,
                    None,
                    group,
                ));
                group_found = true;
            }
        }
        if !group_found {
            for (name, group) in &config.groups {
                // Find the matching group
                if !pattern.matches(name) {
                    continue;
                }
                // Matching group found, collect its trees
                result.append(&mut trees_from_group(
                    app_context,
                    config,
                    graft_config,
                    None,
                    group,
                ));
            }
            if !result.is_empty() {
                return result;
            }
        }
    }

    // No matching gardens or groups were found.
    // Search for matching trees.
    if tree_query.include_trees {
        if syntax::is_graft(query) {
            if let Ok((graft_id, remainder)) = config.get_graft_id(query) {
                result.append(&mut resolve_trees(
                    app_context,
                    config,
                    Some(app_context.get_config(graft_id)),
                    remainder,
                ));
            }
        } else if let Some(graft_cfg) = graft_config {
            result.append(&mut trees(graft_cfg, pattern));
        } else {
            result.append(&mut trees(config, pattern));
        }
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

/// Resolve a tree query into a filtered `Vec<garden::model::TreeContext>`.
///
/// Parameters:
/// - `config`: `&garden::model::Configuration`.
/// - `query`: Tree query `&str`.
/// - `pattern`: Tree name glob pattern used to filter the results.
/// Returns:
/// - `Vec<garden::model::TreeContext>`
pub(crate) fn resolve_and_filter_trees(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    query: &str,
    pattern: &str,
) -> Vec<model::TreeContext> {
    let contexts = resolve_trees(app_context, config, None, query);
    let tree_pattern = glob::Pattern::new(pattern).unwrap_or_default();
    let mut result = Vec::with_capacity(contexts.len());
    for context in contexts {
        if tree_pattern.matches(&context.tree) {
            result.push(context);
        }
    }

    result
}

/// Return tree contexts for every garden matching the specified pattern.
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - pattern: `&glob::Pattern`

fn garden_trees(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    pattern: &glob::Pattern,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();
    let mut garden_found = false;
    if let Some(graft_cfg) = graft_config {
        for (name, garden) in &graft_cfg.gardens {
            if !pattern.matches(name) {
                continue;
            }
            result.append(&mut trees_from_garden(
                app_context,
                config,
                graft_config,
                garden,
            ));
            garden_found = true;
        }
    }
    if !garden_found {
        for (name, garden) in &config.gardens {
            if !pattern.matches(name) {
                continue;
            }
            result.append(&mut trees_from_garden(
                app_context,
                config,
                graft_config,
                garden,
            ));
        }
    }

    result
}

/// Return the tree contexts for a garden
pub fn trees_from_garden(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
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
        let config_groups = match graft_config {
            Some(graft_cfg) => &graft_cfg.groups,
            None => &config.groups,
        };
        for (name, cfg_group) in config_groups {
            if !pattern.matches(name) {
                continue;
            }
            // Match found -- take all of the discovered trees.
            result.append(&mut trees_from_group(
                app_context,
                config,
                graft_config,
                Some(garden.get_name()),
                cfg_group,
            ));
        }
    }

    // Collect tree contexts for each tree in this garden
    for tree in &garden.trees {
        result.append(&mut trees_from_pattern(
            app_context,
            config,
            graft_config,
            tree,
            Some(garden.get_name()),
            None,
        ));
    }

    result
}

/// Return the tree contexts for a garden
pub fn trees_from_group(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    garden: Option<&model::GardenName>,
    group: &model::Group,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();
    for tree in &group.members {
        result.append(&mut trees_from_pattern(
            app_context,
            config,
            graft_config,
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
/// - garden_name: optional name of the garden in which to operate.
/// - group: optional name of the group in which to operate.

pub fn tree_from_name(
    config: &model::Configuration,
    tree_name: &str,
    garden_name: Option<&model::GardenName>,
    group: Option<&model::GroupName>,
) -> Option<model::TreeContext> {
    // Collect tree indexes for the configured trees
    if let Some(tree) = config.trees.get(tree_name) {
        return Some(model::TreeContext::new(
            tree.get_name(),
            config.graft_id(),
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
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    graft_config: Option<&model::Configuration>,
    tree: &str,
    garden_name: Option<&model::GardenName>,
    group: Option<&model::GroupName>,
) -> Vec<model::TreeContext> {
    if syntax::is_graft(tree) {
        // First, try the current config.
        if let Ok((graft_id, remainder)) = config.get_graft_id(tree) {
            return trees_from_pattern(
                app_context,
                config,
                Some(app_context.get_config(graft_id)),
                remainder,
                garden_name,
                group,
            );
        }
    }

    // Collect tree indexes for the configured trees
    let mut result = Vec::new();
    let pattern = match glob::Pattern::new(tree) {
        Ok(value) => value,
        Err(_) => return result,
    };

    if let Some(graft_cfg) = graft_config {
        for (tree_name, cfg_tree) in &graft_cfg.trees {
            if pattern.matches(tree_name) {
                // Tree found in a grafted configuration.
                result.push(model::TreeContext::new(
                    cfg_tree.get_name(),
                    graft_cfg.get_id(),
                    garden_name.cloned(),
                    group.cloned(),
                ));
            }
        }
    }
    for (tree_name, cfg_tree) in &config.trees {
        if pattern.matches(tree_name) {
            // Tree found in a grafted configuration.
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
pub(crate) fn tree_from_path(
    config: &model::Configuration,
    path: &str,
) -> Option<model::TreeContext> {
    tree_from_pathbuf(config, &std::path::PathBuf::from(path))
}

/// Return a tree context for the specified path.
fn tree_from_pathbuf(
    config: &model::Configuration,
    path: &std::path::Path,
) -> Option<model::TreeContext> {
    // First check whether the specified path (including ".") is a configured tree.
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

    // Nothing was found. If "." was specified (or implicitly used) then the
    // current directory is not a configured tree. Fallback to treating "." as
    // the garden config directory to find either configured trees at the root
    // or the implicit default tree when "trees" is omitted.
    let is_dot = path
        .to_str()
        .map(|value| value == constants::DOT)
        .unwrap_or_default();
    if is_dot {
        if let Some(ref dirname) = config.dirname {
            for (name, tree) in &config.trees {
                let tree_path = match tree.path_as_ref() {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let tree_canon = match std::path::PathBuf::from(tree_path).canonicalize() {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                if dirname == &tree_canon {
                    return Some(model::TreeContext::new(name, config.get_id(), None, None));
                }
            }
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

pub(crate) fn tree_name_from_abspath(
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
                config.graft_id(),
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
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    tree: &str,
    garden: Option<&str>,
) -> Result<model::TreeContext, errors::GardenError> {
    let mut ctx = tree_from_name(config, tree, None, None).ok_or_else(|| {
        errors::GardenError::TreeNotFound {
            tree: tree.to_string(),
        }
    })?;
    // If current configuration is a graft then reset the context to the root configuration
    // and record the graft so that later lookups use the graft's context.
    // This is effectively the inverse of the recursive deepening performed by find_tree().
    if config.parent_id.is_some() {
        ctx.config = config.get_id();
    }

    if let Some(garden_name) = garden {
        let pattern = glob::Pattern::new(garden_name).map_err(|_| {
            errors::GardenError::GardenPatternError {
                garden: garden_name.into(),
            }
        })?;
        let contexts = garden_trees(
            app_context,
            app_context.get_root_config(),
            Some(config),
            &pattern,
        );

        if contexts.is_empty() {
            return Err(errors::GardenError::GardenNotFound {
                garden: garden_name.to_string(),
            });
        }

        ctx.garden = garden.map(|value| value.to_string());
        let mut found = false;
        for current_ctx in &contexts {
            if current_ctx.tree == ctx.tree {
                found = true;
                break;
            }
        }

        if !found {
            return Err(errors::GardenError::InvalidGardenArgument {
                tree: tree.to_string(),
                garden: garden_name.to_string(),
            });
        }
    }

    Ok(ctx)
}

pub fn find_tree(
    app_context: &model::ApplicationContext,
    id: model::ConfigId,
    tree: &str,
    garden: Option<&str>,
) -> Result<model::TreeContext, errors::GardenError> {
    {
        let config = app_context.get_config(id);
        if let Some(graft_name) = syntax::graft_basename(tree) {
            if syntax::is_graft(tree) && config.contains_graft(&graft_name) {
                let graft = config.get_graft(&graft_name)?;
                let graft_id = graft
                    .get_id()
                    .ok_or(errors::GardenError::ConfigurationError(format!(
                        "invalid graft: {graft_name}"
                    )))?;
                if let Some(next_graft) = syntax::trim_graft(tree) {
                    return find_tree(app_context, graft_id, &next_graft, garden);
                }
            }
        }
    }

    let config = app_context.get_config(id);
    tree_context(app_context, config, tree, garden)
}

/// Return a path that that is either the tree's path or the tree's shared worktree path.
pub(crate) fn shared_worktree_path(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    ctx: &model::TreeContext,
) -> String {
    let config = match ctx.config {
        Some(config_id) => app_context.get_config(config_id),
        None => config,
    };
    let tree = match config.trees.get(&ctx.tree) {
        Some(tree) => tree,
        None => match app_context.get_root_config().trees.get(&ctx.tree) {
            Some(tree) => tree,
            None => return String::new(),
        },
    };
    if tree.is_worktree {
        let worktree = eval::tree_variable(
            app_context,
            config,
            None,
            &ctx.tree,
            ctx.garden.as_ref(),
            &tree.worktree,
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
