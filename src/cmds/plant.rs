use super::super::cmd;
use super::super::config;
use super::super::errors;
use super::super::git;
use super::super::model;
use super::super::path;
use super::super::query;

use anyhow::Result;
use clap::{Parser, ValueHint};
use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::yaml::Yaml;

// Add pre-existing worktrees to a garden configuration file
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct PlantOptions {
    /// Garden configuration file to write [default: "garden.yaml"]
    #[arg(long, short)]
    output: Option<String>,
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

    // Mutable YAML scope.
    {
        // Get a mutable reference to top-level document hash.
        let doc_hash: &mut YamlHash = match doc {
            Yaml::Hash(ref mut hash) => hash,
            _ => {
                error!("invalid config: not a hash");
            }
        };

        // Get a mutable reference to the "trees" hash.
        let key = Yaml::String("trees".into());
        let trees: &mut YamlHash = match doc_hash.get_mut(&key) {
            Some(Yaml::Hash(ref mut hash)) => hash,
            _ => {
                error!("invalid trees: not a hash");
            }
        };

        for path in &options.paths {
            if let Err(msg) = plant_path(config, verbose, path, trees) {
                error!("{}", msg);
            }
        }
    }

    // Emit the YAML configuration into a string
    Ok(config::writer::write_yaml(&doc, output)?)
}

fn plant_path(
    config: &model::Configuration,
    verbose: u8,
    raw_path: &str,
    trees: &mut YamlHash,
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
    let key = Yaml::String(tree_name.clone());

    // Update an existing tree entry if it already exists.
    // Add a new entry otherwise.
    let mut entry: YamlHash = YamlHash::new();
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
            Yaml::String("worktree".to_string()),
            Yaml::String(parent_tree_name),
        );
        entry.insert(
            Yaml::String("branch".to_string()),
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

    let remotes_key = Yaml::String("remotes".into());
    let has_remotes = match entry.get(&remotes_key) {
        Some(remotes_yaml) => remotes_yaml.as_hash().is_some(),
        None => false,
    };

    // Attempt to get the default remote from "checkout.defaultRemoteName".
    // This can be used to set the default remote name when multiple remotes exist.
    let mut default_remote = "origin".to_string();
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

    if !remotes.is_empty() {
        if !has_remotes {
            entry.insert(remotes_key.clone(), Yaml::Hash(YamlHash::new()));
        }

        let remotes_hash: &mut YamlHash = match entry.get_mut(&remotes_key) {
            Some(Yaml::Hash(ref mut hash)) => hash,
            _ => {
                return Err(
                    errors::GardenError::ConfigurationError(string!("trees: not a hash")).into(),
                );
            }
        };

        for (k, v) in &remotes {
            let remote = Yaml::String(k.clone());
            let value = Yaml::String(v.clone());

            if let Some(remote_entry) = remotes_hash.get_mut(&remote) {
                *remote_entry = value;
            } else {
                remotes_hash.insert(remote, value);
            }
        }
    }

    let url_key = Yaml::String("url".into());
    if verbose > 0 && entry.contains_key(&url_key) {
        eprintln!("{tree_name}: no url");
    }

    // Update the "url" field.
    {
        let remote_url = format!("remote.{default_remote}.url");
        let command = ["git", "config", remote_url.as_str()];
        let exec = cmd::exec_in_dir(&command, &path);
        if let Ok(remote_url) = cmd::stdout_to_string(exec) {
            entry.insert(url_key, Yaml::String(remote_url));
        }
    }

    // Update the "default-remote" field.
    if default_remote != "origin" {
        entry.insert(
            Yaml::String("default-remote".into()),
            Yaml::String(default_remote),
        );
    }

    // Update the "bare" field.
    {
        let bare_key = Yaml::String("bare".into());
        let command = ["git", "config", "--bool", "core.bare"];
        let exec = cmd::exec_in_dir(&command, &path);
        if let Ok(is_bare) = cmd::stdout_to_string(exec) {
            if is_bare == "true" {
                entry.insert(bare_key, Yaml::Boolean(true));
            }
        }
    }

    // Move the entry into the trees container
    if let Some(tree_entry) = trees.get_mut(&key) {
        *tree_entry = Yaml::Hash(entry);
    } else {
        trees.insert(key, Yaml::Hash(entry));
    }

    Ok(())
}
