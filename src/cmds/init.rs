use anyhow::Result;
use clap::{Parser, ValueHint};
use yaml_rust::{yaml, Yaml};

use crate::{cli, cmds::plant, config, constants, errors, git, model, path};

#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct InitOptions {
    /// Do not add any trees when initializing
    #[arg(long)]
    pub empty: bool,
    /// Overwrite existing config files
    #[arg(long, short)]
    pub force: bool,
    /// Use the user-wide configuration directory (~/.config/garden/garden.yaml)
    #[arg(long)]
    pub global: bool,
    /// Set the garden root path
    #[arg(long, default_value_t = string!(constants::GARDEN_CONFIG_DIR_EXPR), value_hint = ValueHint::DirPath)]
    pub root: String,
    /// Config filename to write
    #[arg(default_value = constants::GARDEN_CONFIG, value_hint = ValueHint::FilePath)]
    pub filename: std::path::PathBuf,
}

pub fn main(options: &cli::MainOptions, init_options: &mut InitOptions) -> Result<()> {
    let mut dirname = path::current_dir();
    let file_path = &init_options.filename;
    if file_path.is_absolute() {
        if init_options.global {
            return Err(errors::GardenError::Usage(
                "'--global' cannot be used with an absolute path".into(),
            )
            .into());
        }

        dirname = file_path
            .parent()
            .ok_or_else(|| {
                errors::GardenError::AssertionError(format!(
                    "unable to get parent(): {file_path:?}"
                ))
            })?
            .to_path_buf();

        init_options.filename =
            std::path::PathBuf::from(file_path.file_name().ok_or_else(|| {
                errors::GardenError::AssertionError(format!(
                    "unable to get file path: {file_path:?}"
                ))
            })?);
    }
    if init_options.global {
        dirname = config::xdg_dir();
    }

    let mut config_path = dirname.clone();
    config_path.push(&init_options.filename);

    if !init_options.force && config_path.exists() {
        let error_message = format!(
            "{:?} already exists, use \"--force\" to overwrite",
            config_path.to_string_lossy()
        );
        return Err(errors::GardenError::FileExists(error_message).into());
    }

    // Create parent directories as needed
    let parent = config_path
        .parent()
        .ok_or_else(|| {
            errors::GardenError::AssertionError(format!("unable to get parent(): {config_path:?}"))
        })?
        .to_path_buf();

    if !parent.exists() {
        if let Err(err) = std::fs::create_dir_all(&parent) {
            let error_message = format!("unable to create {parent:?}: {err}");
            return Err(errors::GardenError::OSError(error_message).into());
        }
    }

    // Does the config file already exist?
    let exists = config_path.exists();

    // Read or create a new document
    let mut doc = if exists {
        config::reader::read_yaml(&config_path)?
    } else {
        config::reader::empty_doc()
    };

    let mut config = model::Configuration::new();
    config.root = model::Variable::new(init_options.root.clone(), None);
    config.root_path.clone_from(&dirname);
    config.path = Some(config_path.clone());

    let mut done = false;
    if !init_options.empty && init_options.root == constants::GARDEN_CONFIG_DIR_EXPR {
        let git_worktree = git::current_worktree_path(&dirname);
        if let Ok(worktree) = git_worktree {
            config::reader::add_section(constants::TREES, &mut doc)?;
            if let Yaml::Hash(ref mut doc_hash) = doc {
                let trees_key = Yaml::String(constants::TREES.into());
                if let Some(Yaml::Hash(trees)) = doc_hash.get_mut(&trees_key) {
                    if let Ok(tree_name) =
                        plant::plant_path(None, &config, options.verbose, &worktree, trees)
                    {
                        done = true;
                        // If the config path is the same as the tree's worktree path then
                        // set the tree's "path" field to ${GARDEN_CONFIG_DIR}.
                        if config.root_path.to_string_lossy() == worktree {
                            if let Some(Yaml::Hash(tree_entry)) = trees.get_mut(&tree_name) {
                                tree_entry.insert(
                                    Yaml::String(constants::PATH.to_string()),
                                    Yaml::String(constants::GARDEN_CONFIG_DIR_EXPR.to_string()),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Mutable scope
    if !done || init_options.root != constants::GARDEN_CONFIG_DIR_EXPR {
        config::reader::add_section(constants::GARDEN, &mut doc)?;
        if let Yaml::Hash(ref mut doc_hash) = doc {
            let garden_key = Yaml::String(constants::GARDEN.into());
            let garden: &mut yaml::Hash = match doc_hash.get_mut(&garden_key) {
                Some(Yaml::Hash(ref mut hash)) => hash,
                _ => {
                    return Err(errors::GardenError::InvalidConfiguration {
                        msg: "invalid configuration: 'garden' is not a hash".into(),
                    }
                    .into());
                }
            };

            let root_key = Yaml::String(constants::ROOT.into());
            garden.insert(root_key, Yaml::String(init_options.root.clone()));
        }
    }

    config::writer::write_yaml(&doc, &config_path)?;

    if !options.quiet {
        if exists {
            eprintln!("Reinitialized Garden configuration in {config_path:?}");
        } else {
            eprintln!("Initialized Garden configuration in {config_path:?}");
        }
    }

    Ok(())
}
