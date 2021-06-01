use anyhow::Result;
use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::yaml::Yaml;

use super::super::cmd;
use super::super::config;
use super::super::errors;
use super::super::model;
use super::super::path;

struct InitOptions {
    pub dirname: std::path::PathBuf,
    pub filename: String,
    pub force: bool,
    pub global: bool,
    pub root: String,
}

impl std::default::Default for InitOptions {
    fn default() -> Self {
        InitOptions {
            dirname: path::current_dir(),
            filename: "garden.yaml".to_string(),
            force: false,
            global: false,
            root: "${GARDEN_CONFIG_DIR}".to_string(),
        }
    }
}

pub fn main(options: &mut model::CommandOptions) -> Result<()> {
    let mut init_options = InitOptions::default();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden init - create an empty garden.yaml");

        ap.refer(&mut init_options.global).add_option(
            &["--global"],
            argparse::StoreTrue,
            "use the user-wide configuration directory
                        (~/.config/garden/garden.yaml)",
        );

        ap.refer(&mut init_options.force).add_option(
            &["-f", "--force"],
            argparse::StoreTrue,
            "overwrite existing config files",
        );

        ap.refer(&mut init_options.root).add_option(
            &["-r", "--root"],
            argparse::Store,
            "specify the garden root
                        (default: ${GARDEN_CONFIG_DIR}",
        );

        ap.refer(&mut init_options.filename).add_argument(
            "filename",
            argparse::Store,
            "config file to write (default: garden.yaml)",
        );

        options.args.insert(0, "garden init".into());
        cmd::parse_args(ap, options.args.to_vec());
    }

    init(options, &mut init_options)
}

fn init(options: &model::CommandOptions, init_options: &mut InitOptions) -> Result<()> {
    let file_path = std::path::PathBuf::from(&init_options.filename);
    if file_path.is_absolute() {
        if init_options.global {
            return Err(errors::GardenError::Usage(
                "'--global' cannot be used with an absolute path".into(),
            )
            .into());
        }

        init_options.dirname = file_path
            .parent()
            .as_ref()
            .ok_or_else(|| {
                errors::GardenError::AssertionError(format!(
                    "unable to get parent(): {:?}",
                    file_path
                ))
            })?
            .to_path_buf();

        init_options.filename = file_path
            .file_name()
            .as_ref()
            .ok_or_else(|| {
                errors::GardenError::AssertionError(format!(
                    "unable to get file path: {:?}",
                    file_path
                ))
            })?
            .to_string_lossy()
            .to_string();
    }
    if init_options.global {
        init_options.dirname = config::xdg_dir();
    }

    let mut config_path = init_options.dirname.to_path_buf();
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
        .as_ref()
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
    let mut doc;
    if exists {
        doc = config::reader::read_yaml(&config_path)?;
    } else {
        doc = config::reader::empty_doc();
    }

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
