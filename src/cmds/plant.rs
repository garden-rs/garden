use anyhow::Result;
use clap::{Parser, ValueHint};
use yaml_rust::{yaml, Yaml};

use crate::{cmd, config, constants, errors, eval, git, model, path, query};

// Add pre-existing worktrees to a garden configuration file
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct PlantOptions {
    /// Garden configuration file to write [default: "garden.yaml"]
    #[arg(long, short)]
    output: Option<String>,
    /// Sort all trees after planting new trees
    #[arg(long, short)]
    sort: bool,
    /// Trees to plant
    #[arg(required = true, value_hint=ValueHint::DirPath)]
    paths: Vec<String>,
}

pub fn main(app_context: &model::ApplicationContext, options: &PlantOptions) -> Result<()> {
    // Read existing configuration
    let verbose = app_context.options.verbose;
    let config = app_context.get_root_config();
    let mut doc = config::reader::read_yaml(config.get_path()?)?;

    // Output filename defaults to the input filename.
    let output = match &options.output {
        Some(output) => output.to_string(),
        None => config.get_path()?.to_string_lossy().to_string(),
    };
    let trees_key = Yaml::String(constants::TREES.to_string());
    config::reader::add_section(constants::TREES, &mut doc)?;

    // Mutable YAML scope.
    {
        // Get a mutable reference to top-level document hash.
        let doc_hash: &mut yaml::Hash = match doc {
            Yaml::Hash(ref mut hash) => hash,
            _ => {
                error!("invalid config: not a hash");
            }
        };
        // Get a mutable reference to the "trees" hash.
        let trees: &mut yaml::Hash = match doc_hash.get_mut(&trees_key) {
            Some(Yaml::Hash(ref mut hash)) => hash,
            _ => {
                error!("invalid trees: not a hash");
            }
        };
        for path in &options.paths {
            if let Err(msg) = plant_path(Some(app_context), config, verbose, path, trees) {
                error!("{}", msg);
            }
        }
    }

    if options.sort {
        // Get a mutable reference to top-level document hash.
        let doc_hash: &mut yaml::Hash = match doc {
            Yaml::Hash(ref mut hash) => hash,
            _ => {
                error!("invalid config: not a hash");
            }
        };
        // Gather and clone trees in a read-only scope.
        let mut names_and_trees = Vec::new();
        {
            let trees: &yaml::Hash = match doc_hash.get(&trees_key) {
                Some(Yaml::Hash(hash)) => hash,
                _ => {
                    error!("invalid trees: not a hash");
                }
            };
            for (k, v) in trees {
                if let Yaml::String(tree_name) = k {
                    names_and_trees.push((tree_name.clone(), v.clone()));
                }
            }
        }
        // Sort trees case-insensitively.
        names_and_trees.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        // Build a new trees table and replace the existing entry with it.
        let mut sorted_trees = yaml::Hash::new();
        for (name, tree) in names_and_trees {
            sorted_trees.insert(Yaml::String(name), tree);
        }
        if let Some(trees) = doc_hash.get_mut(&trees_key) {
            *trees = Yaml::Hash(sorted_trees);
        }
    }

    // Emit the YAML configuration into a string
    Ok(config::writer::write_yaml(&doc, output)?)
}

pub(crate) fn plant_path(
    app_context: Option<&model::ApplicationContext>,
    config: &model::Configuration,
    verbose: u8,
    raw_path: &str,
    trees: &mut yaml::Hash,
) -> Result<()> {
    // Garden root path
    let root = config.root_path.canonicalize().map_err(|err| {
        errors::GardenError::ConfigurationError(format!(
            "unable to canonicalize config root: {err:?}"
        ))
    })?;

    let pathbuf = std::path::PathBuf::from(raw_path);
    if !pathbuf.exists() {
        return Err(errors::GardenError::ConfigurationError(format!(
            "invalid tree path: {raw_path}"
        ))
        .into());
    }

    let mut is_worktree = false;
    let mut parent_tree_name = String::new();
    let worktree_details = git::worktree_details(&pathbuf)?;

    // If this is a worktree child then automatically "garden plant" the parent worktree.
    if let model::GitTreeType::Worktree(parent_path) = worktree_details.tree_type {
        is_worktree = true;

        parent_tree_name = match query::tree_name_from_abspath(config, &parent_path) {
            Some(tree_name) => tree_name,
            None => {
                let relative_path = path::strip_prefix(&root, &parent_path)?;
                return Err(errors::GardenError::WorktreeParentNotPlantedError {
                    parent: relative_path,
                    tree: raw_path.into(),
                }
                .into());
            }
        };
    }

    // Get a canonical tree path for comparison with the canonical root.
    let path = pathbuf.canonicalize().map_err(|err| {
        errors::GardenError::ConfigurationError(format!(
            "unable to canonicalize {raw_path:?}: {err:?}"
        ))
    })?;

    // Build the tree's path
    let tree_path = path::strip_prefix_into_string(&root, &path)?;

    // Tree name is updated when an existing tree is found.
    let tree_name = match query::tree_name_from_abspath(config, &path) {
        Some(value) => value,
        None => tree_path,
    };

    // Key for the tree entry
    let mut key = Yaml::String(tree_name.clone());

    // Update an existing tree entry if it already exists.
    // Add a new entry otherwise.
    let mut entry: yaml::Hash = yaml::Hash::new();
    if let Some(tree_yaml) = trees.get(&key) {
        if let Some(tree_hash) = tree_yaml.as_hash() {
            if verbose > 0 {
                eprintln!("{tree_name}: found existing tree");
            }
            entry = tree_hash.clone();
        }
    }

    // If this is a child worktree then record a "worktree" entry only.
    if is_worktree {
        entry.insert(
            Yaml::String(constants::WORKTREE.to_string()),
            Yaml::String(parent_tree_name),
        );
        entry.insert(
            Yaml::String(constants::BRANCH.to_string()),
            Yaml::String(worktree_details.branch.to_string()),
        );

        // Move the entry into the trees container
        if let Some(tree_entry) = trees.get_mut(&key) {
            *tree_entry = Yaml::Hash(entry);
        } else {
            trees.insert(key, Yaml::Hash(entry));
        }

        return Ok(());
    }

    let remotes_key = Yaml::String(constants::REMOTES.into());
    let has_remotes = match entry.get(&remotes_key) {
        Some(remotes_yaml) => remotes_yaml.as_hash().is_some(),
        None => false,
    };

    // Attempt to get the default remote from "checkout.defaultRemoteName".
    // This can be used to set the default remote name when multiple remotes exist.
    let mut default_remote = constants::ORIGIN.to_string();
    let command = ["git", "config", "checkout.defaultRemoteName"];
    let exec = cmd::exec_in_dir(&command, &path);
    if let Ok(output) = cmd::stdout_to_string(exec) {
        // If only a single remote exists then capture its name.
        if !output.is_empty() {
            default_remote = output.to_string();
        }
    }

    // Gather remote names.
    let mut remote_names: Vec<String> = Vec::new();
    {
        let command = ["git", "remote"];
        let exec = cmd::exec_in_dir(&command, &path);
        if let Ok(output) = cmd::stdout_to_string(exec) {
            // We have to do this in two passes to detect the scenario where only a
            // single remote exists and its name is *not* "origin".
            // If only a single remote exists then capture its name.
            if !output.is_empty() && output.lines().count() == 1 {
                default_remote = output.to_string();
            }

            for line in output.lines() {
                // Skip the default remote since it is defined by the "url" entry.
                if line == default_remote {
                    continue;
                }
                // Any other remotes are part of the "remotes" hash.
                remote_names.push(line.into());
            }
        }
    }

    // Gather remote urls
    let mut remotes: Vec<(String, String)> = Vec::new();
    {
        for remote in &remote_names {
            let cmd = ["git", "config", &format!("remote.{remote}.url")];
            let exec = cmd::exec_in_dir(&cmd, &path);
            if let Ok(output) = cmd::stdout_to_string(exec) {
                remotes.push((remote.clone(), output));
            }
        }
    }

    let mut is_empty = false;
    if !remotes.is_empty() {
        if !has_remotes {
            entry.insert(remotes_key.clone(), Yaml::Hash(yaml::Hash::new()));
        }

        let remotes_hash: &mut yaml::Hash = match entry.get_mut(&remotes_key) {
            Some(Yaml::Hash(ref mut hash)) => hash,
            _ => {
                return Err(errors::GardenError::ConfigurationError(string!(
                    "remotes: not a hash"
                ))
                .into());
            }
        };

        for (remote_str, value_str) in &remotes {
            if let Some(current_value) =
                app_context.and_then(|ctx| get_url_for_remote(ctx, config, &tree_name, remote_str))
            {
                // Leave existing remotes as-is if their evaluated value
                // resolves to the value from git.
                if &current_value == value_str {
                    continue;
                }
            }
            let remote = Yaml::String(remote_str.clone());
            let value = Yaml::String(value_str.clone());
            if let Some(remote_entry) = remotes_hash.get_mut(&remote) {
                *remote_entry = value;
            } else {
                remotes_hash.insert(remote, value);
            }
        }
        is_empty = remotes_hash.is_empty();
    }

    // A template might not have remotes so we should purge the empty "remotes:" block.
    if is_empty {
        entry.remove(&remotes_key);
    }

    let url_key = Yaml::String(constants::URL.into());
    if verbose > 0 && !entry.contains_key(&url_key) {
        eprintln!("{tree_name}: no url");
    }

    // Update the "url" field.
    let mut update_url = true;
    let mut url = String::new();
    {
        let remote_url = format!("remote.{default_remote}.url");
        let command = ["git", "config", remote_url.as_str()];
        let exec = cmd::exec_in_dir(&command, &path);
        if let Ok(remote_url) = cmd::stdout_to_string(exec) {
            url = remote_url.clone();
            if let Some(current_url) = app_context
                .and_then(|ctx| get_url_for_remote(ctx, config, &tree_name, &default_remote))
            {
                // Leave existing remotes as-is if their evaluated value
                // resolves to the value from git.
                update_url = current_url != remote_url;
            }
            if update_url {
                entry.insert(url_key.clone(), Yaml::String(remote_url));
            }
        }
    }

    // Update the "default-remote" field.
    if default_remote != constants::ORIGIN {
        entry.insert(
            Yaml::String(constants::DEFAULT_REMOTE.into()),
            Yaml::String(default_remote),
        );
    }

    // Update the "bare" field.
    {
        let bare_key = Yaml::String(constants::BARE.into());
        let command = ["git", "config", "--bool", "core.bare"];
        let exec = cmd::exec_in_dir(&command, &path);
        if let Ok(is_bare) = cmd::stdout_to_string(exec) {
            if is_bare == "true" {
                entry.insert(bare_key, Yaml::Boolean(true));
            }
        }
    }

    // Parse the tree name from the URL.
    if tree_name.is_empty() {
        let tree_name = git::name_from_url_or_path(&url, &path);
        key = Yaml::String(tree_name);
    }

    // Move the entry into the trees container
    if let Some(tree_entry) = trees.get_mut(&key) {
        // The entry can be empty if we ended up not actually changing anything.
        if !entry.is_empty() {
            let is_string = matches!(tree_entry, Yaml::String(_));
            let is_still_string = entry.len() == 1 && entry.contains_key(&url_key);
            if is_string && is_still_string {
                *tree_entry = Yaml::String(url);
            } else if is_string && !is_still_string && !update_url {
                entry.insert(url_key, tree_entry.clone());
                *tree_entry = Yaml::Hash(entry);
            } else {
                *tree_entry = Yaml::Hash(entry);
            }
        }
    } else {
        trees.insert(key, Yaml::Hash(entry));
    }

    Ok(())
}

/// Return the currently configured evaluated value for a git remote.
fn get_url_for_remote(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    tree_name: &str,
    remote: &str,
) -> Option<String> {
    let tree = config.trees.get(tree_name)?;
    let remote_variable = tree.remotes.get(remote)?;
    let current_value =
        eval::tree_variable(app_context, config, None, tree_name, None, remote_variable);

    Some(current_value)
}
