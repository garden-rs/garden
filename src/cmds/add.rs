use anyhow::Result;
use yaml_rust::yaml::Yaml;
use yaml_rust::yaml::Hash as YamlHash;

use super::super::cmd;
use super::super::config;
use super::super::model;


pub fn main(app: &mut model::ApplicationContext) -> Result<()> {
    // Parse arguments
    let options = &mut app.options;
    let config = &mut app.config;

    let mut output = String::new();
    let mut paths: Vec<String> = Vec::new();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("add existing trees to a garden configuration");

        ap.refer(&mut output)
            .add_option(&["-o", "--output"], argparse::Store,
                        "file to write (defaults to the config file)");

        ap.refer(&mut paths).required()
            .add_argument("paths", argparse::List, "trees to add");

        options.args.insert(0, "garden add".to_string());
        cmd::parse_args(ap, options.args.to_vec());
    }

    // Read existing configuration
    let mut doc = config::reader::read_yaml(config.path.as_ref().unwrap())?;

    // Output filename defaults to the input filename.
    if output.is_empty() {
        output = config.path.as_ref().unwrap().to_string_lossy().to_string();
    }

    {
        // Get a mutable reference to top-level document hash.
        let doc_hash: &mut YamlHash = match doc {
            Yaml::Hash(ref mut hash) => {
                hash
            },
            _ => {
                error!("invalid config: not a hash");
            },
        };

        // Get a mutable reference to the "trees" hash.
        let key = Yaml::String("trees".to_string());
        let trees: &mut YamlHash = match doc_hash.get_mut(&key) {
            Some(Yaml::Hash(ref mut hash)) => {
                hash
            },
            _ => {
                error!("invalid trees: not a hash");
            }
        };

        for path in &paths {
            if let Err(msg) = add_path(config, options.verbose, path, trees) {
                error!("{}", msg);
            }
        }
    }

    // Emit the YAML configuration into a string
    Ok(config::writer::write_yaml(&doc, &output)?)
}


fn add_path(
    config: &model::Configuration,
    verbose: bool,
    raw_path: &str,
    trees: &mut YamlHash)
-> Result<(), String> {

    // Garden root path
    let root = config.root_path.canonicalize().unwrap().to_path_buf();

    let pathbuf = std::path::PathBuf::from(raw_path);
    if !pathbuf.exists() {
        return Err(format!("{}: invalid tree path", raw_path));
    }

    // Build the tree's path
    let tree_path: String;

    // Get a canonical tree path for comparison with the canonical root.
    let path = pathbuf.canonicalize().unwrap().to_path_buf();

    // Is the path a child of the current garden root?
    if path.starts_with(&root) && path.strip_prefix(&root).is_ok() {

        tree_path = path
            .strip_prefix(&root).unwrap().to_string_lossy().to_string();
    } else {
        tree_path = path.to_string_lossy().to_string();
    }

    // Tree name is updated when an existing tree is found.
    let mut tree_name = tree_path.to_string();

    // Do we already have a tree with this tree path?
    for tree in &config.trees {
        let cfg_tree_path_result = std::path::PathBuf::from(
            tree.path.value.as_ref().unwrap()).canonicalize();
        if cfg_tree_path_result.is_err() {
            continue;  // skip missing entries
        }

        let cfg_tree_path = cfg_tree_path_result.unwrap();
        if cfg_tree_path == path {
            // Tree found: take its configured name.
            tree_name = tree.name.to_string();
        }
    }

    // Key for the tree entry
    let key = Yaml::String(tree_name.to_string());
    let mut entry: YamlHash = YamlHash::new();

    // Update an existing entry if it already exists.
    // Add a new entry otherwise.
    if trees.get(&key).is_some()
    && trees.get(&key).unwrap().as_hash().is_some() {
        entry = trees.get(&key).unwrap().as_hash().unwrap().clone();
        if verbose {
            eprintln!("{}: found existing tree", tree_name);
        }
    }

    let remotes_key = Yaml::String("remotes".to_string());
    let has_remotes = entry.contains_key(&remotes_key)
        && entry.get(&remotes_key).unwrap().as_hash().is_some();

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
                remote_names.push(line.to_string());
            }
        }
    }

    // Gather remote urls
    let mut remotes: Vec<(String, String)> = Vec::new();
    {
        for remote in &remote_names {
            let mut command: Vec<String> = Vec::new();
            command.push("git".into());
            command.push("config".into());
            command.push("remote.".to_string() + remote + ".url");

            let exec = cmd::exec_in_dir(&command, &path);
            if let Ok(x) = cmd::capture_stdout(exec) {
                let output = cmd::trim_stdout(&x);
                remotes.push((remote.to_string(), output));
            }
        }
    }

    if !remotes.is_empty() {
        if !has_remotes {
            entry.insert(remotes_key.clone(), Yaml::Hash(YamlHash::new()));
        }

        let remotes_hash: &mut YamlHash =
            match entry.get_mut(&remotes_key) {
            Some(Yaml::Hash(ref mut hash)) => {
                hash
            },
            _ => {
                return Err("trees: not a hash".to_string());
            }
        };

        for (k, v) in &remotes {
            let remote = Yaml::String(k.to_string());
            let value = Yaml::String(v.to_string());

            if remotes_hash.contains_key(&remote) {
                *(remotes_hash.get_mut(&remote).unwrap()) = value;
            } else {
                remotes_hash.insert(remote, value);
            }
        }
    }

    let url_key = Yaml::String("url".to_string());
    let has_url = entry.contains_key(&url_key);
    if !has_url {
        if verbose {
            eprintln!("{}: no url", tree_name);
        }

        let command = ["git", "config", "remote.origin.url"];
        let exec = cmd::exec_in_dir(&command, &path);
        match cmd::capture_stdout(exec) {
            Ok(x) => {
                let origin_url = cmd::trim_stdout(&x);
                entry.insert(url_key, Yaml::String(origin_url));
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
    }

    // Move the entry into the trees container
    if trees.contains_key(&key) {
        *(trees.get_mut(&key).unwrap()) = Yaml::Hash(entry);
    } else {
        trees.insert(key, Yaml::Hash(entry));
    }

    Ok(())
}
