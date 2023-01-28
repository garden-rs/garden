use anyhow::Result;
use clap::{Parser, ValueHint};

use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::yaml::Yaml;

use super::super::cli;
use super::super::config;
use super::super::errors;
use super::super::path;

#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct InitOptions {
    /// Overwrite existing config files
    #[arg(long, short)]
    pub force: bool,
    /// Use the user-wide configuration directory (~/.config/garden/garden.yaml)
    #[arg(long)]
    pub global: bool,
    /// Set the garden root path
    #[arg(long, default_value_t = String::from("${GARDEN_CONFIG_DIR}"), value_hint = ValueHint::DirPath)]
    pub root: String,
    /// Config filename to write
    #[arg(default_value = "garden.yaml", value_hint = ValueHint::FilePath)]
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
                    "unable to get parent(): {:?}",
                    file_path
                ))
            })?
            .to_path_buf();

        init_options.filename =
            std::path::PathBuf::from(file_path.file_name().ok_or_else(|| {
                errors::GardenError::AssertionError(format!(
                    "unable to get file path: {:?}",
                    file_path
                ))
            })?);
    }
    if init_options.global {
        dirname = config::xdg_dir();
    }

    let mut config_path = dirname;
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
            errors::GardenError::AssertionError(format!(
                "unable to get parent(): {:?}",
                config_path
            ))
        })?
        .to_path_buf();

    if !parent.exists() {
        if let Err(err) = std::fs::create_dir_all(&parent) {
            let error_message = format!("unable to create {:?}: {}", parent, err);
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

    // Mutable scope
    {
        if let Yaml::Hash(ref mut doc_hash) = doc {
            let garden_key = Yaml::String("garden".into());
            let garden: &mut YamlHash = match doc_hash.get_mut(&garden_key) {
                Some(Yaml::Hash(ref mut hash)) => hash,
                _ => {
                    return Err(errors::GardenError::InvalidConfiguration {
                        msg: "invalid configuration: 'garden' is not a hash".into(),
                    }
                    .into());
                }
            };

            let root_key = Yaml::String("root".into());
            garden.insert(root_key, Yaml::String(init_options.root.clone()));
        }
    }

    config::writer::write_yaml(&doc, &config_path)?;

    if !options.quiet {
        if exists {
            eprintln!("Reinitialized Garden configuration in {:?}", config_path);
        } else {
            eprintln!(
                "Initialized empty Garden configuration in {:?}",
                config_path
            );
        }
    }

    Ok(())
}
