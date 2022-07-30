use anyhow::Result;
use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::yaml::Yaml;

use super::super::cmd;
use super::super::config;
use super::super::errors::GardenError;
use super::super::model;

pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    let mut output = String::new();
    let mut paths: Vec<String> = Vec::new();
    parse_args(&mut app.options, &mut output, &mut paths);

    // Read existing configuration
    let verbose = app.options.verbose;
    let config = app.get_root_config_mut();
    let mut doc = config::reader::read_yaml(config.get_path()?)?;

    // Output filename defaults to the input filename.
    if output.is_empty() {
        output = config.get_path()?.to_string_lossy().into();
    }

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

        for path in &paths {
            if let Err(msg) = plant_path(config, verbose, path, trees) {
                error!("{}", msg);
            }
        }
    }

    // Emit the YAML configuration into a string
    Ok(config::writer::write_yaml(&doc, &output)?)
}

fn parse_args(options: &mut model::CommandOptions, output: &mut String, paths: &mut Vec<String>) {
    let mut ap = argparse::ArgumentParser::new();
    ap.set_description("add pre-existing worktrees to a garden configuration");

    ap.refer(output).add_option(
        &["-o", "--output"],
        argparse::Store,
        "file to write (defaults to the config file)",
    );

    ap.refer(paths)
        .required()
        .add_argument("paths", argparse::List, "trees to add");

    options.args.insert(0, "garden plant".into());
    cmd::parse_args(ap, options.args.to_vec());
}

fn plant_path(
    config: &model::Configuration,
    verbose: u8,
    raw_path: &str,
    trees: &mut YamlHash,
) -> Result<()> {
    // Garden root path
    let root = config.root_path.canonicalize().map_err(|err| {
        GardenError::ConfigurationError(format!("unable to canonicalize config root: {:?}", err))
    })?;

    let pathbuf = std::path::PathBuf::from(raw_path);
    if !pathbuf.exists() {
        return Err(
            GardenError::ConfigurationError(format!("invalid tree path: {}", raw_path)).into(),
        );
    }

    // Build the tree's path
    let tree_path: String;

    // Get a canonical tree path for comparison with the canonical root.
    let path = pathbuf.canonicalize().map_err(|err| {
        GardenError::ConfigurationError(format!("unable to canonicalize {:?}: {:?}", raw_path, err))
    })?;

    // Is the path a child of the current garden root?
    if path.starts_with(&root) {
        tree_path = path
            .strip_prefix(&root)
            .map_err(|err| {
                GardenError::ConfigurationError(format!(
                    "{:?} is not a child of {:?}: {:?}",
                    path, root, err
                ))
            })?
            .to_string_lossy()
            .into();
    } else {
        tree_path = path.to_string_lossy().into();
    }

    // Tree name is updated when an existing tree is found.
    let mut tree_name = tree_path;

    // Do we already have a tree with this tree path?
    for tree in &config.trees {
        assert!(tree.path_is_valid());
        // Skip entries that do not exist on disk.
        // Check if this tree matches the specified path.
        let tree_path_value = tree.path_as_ref()?;
        let tree_pathbuf = std::path::PathBuf::from(tree_path_value);
        if let Ok(canon_path) = tree_pathbuf.canonicalize() {
            if canon_path == path {
                // Existing tree found: use the configured name.
                tree_name = tree.get_name().to_string();
                break;
            }
        }
    }

    // Key for the tree entry
    let key = Yaml::String(tree_name.clone());

    // Update an existing tre entry if it already exists.
    // Add a new entry otherwise.
    let mut entry: YamlHash = YamlHash::new();
    if let Some(tree_yaml) = trees.get(&key) {
        if let Some(tree_hash) = tree_yaml.as_hash() {
            if verbose > 0 {
                eprintln!("{}: found existing tree", tree_name);
            }
            entry = tree_hash.clone();
        }
    }

    let remotes_key = Yaml::String("remotes".into());
    let has_remotes = match entry.get(&remotes_key) {
        Some(remotes_yaml) => remotes_yaml.as_hash().is_some(),
        None => false,
    };

    // Gather remote names
    let mut remote_names: Vec<String> = Vec::new();
    {
        let command = ["git", "remote"];
        let exec = cmd::exec_in_dir(&command, &path);
        if let Ok(x) = cmd::capture_stdout(exec) {
            let output = cmd::trim_stdout(&x);

            for line in output.lines() {
                // Skip "origin" since it is defined by the "url" entry.
                if line == "origin" {
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
            let command: Vec<String> = vec![
                "git".into(),
                "config".into(),
                "remote.".to_string() + remote + ".url",
            ];

            let exec = cmd::exec_in_dir(&command, &path);
            if let Ok(x) = cmd::capture_stdout(exec) {
                let output = cmd::trim_stdout(&x);
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
                    GardenError::ConfigurationError("trees: not a hash".to_string()).into(),
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
        eprintln!("{}: no url", tree_name);
    }

    // Update the "url" field.
    {
        let command = ["git", "config", "remote.origin.url"];
        let exec = cmd::exec_in_dir(&command, &path);
        if let Ok(cmd_stdout) = cmd::capture_stdout(exec) {
            let origin_url = cmd::trim_stdout(&cmd_stdout);
            entry.insert(url_key, Yaml::String(origin_url));
        }
    }

    // Update the "bare" field.
    {
        let bare_key = Yaml::String("bare".into());
        let command = ["git", "config", "--bool", "core.bare"];
        let exec = cmd::exec_in_dir(&command, &path);
        if let Ok(cmd_stdout) = cmd::capture_stdout(exec) {
            let is_bare = cmd::trim_stdout(&cmd_stdout);
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
