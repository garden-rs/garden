extern crate dirs;
extern crate xdg;

use ::model;
use ::config_yaml;

// Search for configuration in the following locations:
//  .
//  ./garden
//  ./etc/garden
//  ~/.config/garden
//  ~/etc/garden
//  /etc/garden

fn search_path() -> Vec<std::path::PathBuf> {
    // Result: Vec<PathBufs> in priority order
    let mut paths: Vec<std::path::PathBuf> = Vec::new();

    let current_dir = std::env::current_dir().unwrap();
    let home_dir = dirs::home_dir().unwrap();

    // . Current directory
    paths.push(current_dir.to_path_buf());

    // ./garden
    let mut current_garden_dir  = current_dir.to_path_buf();
    current_garden_dir.push("garden");
    if current_garden_dir.exists() {
        paths.push(current_garden_dir);
    }

    // ./etc/garden
    let mut current_etc_garden_dir = current_dir.to_path_buf();
    current_etc_garden_dir.push("etc");
    current_etc_garden_dir.push("garden");
    if current_etc_garden_dir.exists() {
        paths.push(current_etc_garden_dir);
    }

    // $XDG_CONFIG_HOME/garden (typically ~/.config/garden)
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let mut home_config_dir = xdg_dirs.get_config_home().to_path_buf();
    home_config_dir.push("garden");
    if home_config_dir.exists() {
        paths.push(home_config_dir);
    }

    // ~/etc/garden
    let mut home_etc_dir = home_dir.to_path_buf();
    home_etc_dir.push("etc");
    home_etc_dir.push("garden");
    if home_etc_dir.exists() {
        paths.push(home_etc_dir);
    }

    // /etc/garden
    let etc_garden = std::path::PathBuf::from("/etc/garden");
    if etc_garden.exists() {
        paths.push(etc_garden);
    }

    paths
}


pub fn new(config: &Option<std::path::PathBuf>, verbose: bool)
-> model::Configuration {

    let mut cfg = model::Configuration::new();
    cfg.verbose = verbose;

    let mut basename = "garden".to_string();

    // Find garden.yaml in the search path
    let mut found = false;
    if let Some(config_path) = config {
        if config_path.is_file() || config_path.is_absolute() {
            // If an absolute apath was specified, or if the file exists, short-circuit
            // the search; the config file is missing so we shouldn't silently use a
            // different config files.
            cfg.path = Some(config_path.to_path_buf());
            found = true;
        } else {
            // The config path can be a basename that will be found in the config path.
            if config_path.is_relative() {
                if config_path.extension().is_some() {
                    // Convenience: if the user specified a filename with an
                    // extension then strip off the extension so that we have
                    // just the basename.  Valid extensions are appended when
                    // traversing the search path.
                    //
                    // Technically the user could specify e.g. "foo.txt" and
                    // we would only search for "foo.yaml", but that's fine
                    // since it's an unsupported use case.
                    //
                    // Config files must have a .yaml or .json extension.
                    basename = config_path
                        .file_stem().unwrap().to_string_lossy().to_string();
                } else {
                    // The user specified a plain basename -> use it as-is.
                    basename = config_path.to_string_lossy().to_string();
                }
            }
        }
    }

    if !found {
        for entry in search_path() {
            let formats = vec!("yaml", "json");
            for fmt in &formats {
                let mut candidate = entry.to_path_buf();
                candidate.push(basename.to_string() + "." + fmt);
                if candidate.exists() {
                    cfg.path = Some(candidate);
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
    }
    if verbose {
        debug!("config path is {:?}{}", cfg.path,
               match found {
                   true => "",
                   false => " (NOT FOUND)",
               });
    }

    if found {
        // Read file contents
        let config_string = unwrap_or_err_return!(
            std::fs::read_to_string(cfg.path.as_ref().unwrap()),
            cfg, "unable to read {:?}: {}", cfg.path.as_ref().unwrap());
        parse(&config_string, verbose, &mut cfg);
    }

    return cfg;
}


/// Create a model::Configuration instance from model::CommandOptions

pub fn from_options(options: &model::CommandOptions) -> model::Configuration {
    let config_verbose = options.is_debug("config::new");
    let mut config = new(&options.filename, config_verbose);
    if config.path.is_none() {
        error!("unable to find a configuration file -- use --config <path>");
    }
    if options.is_debug("config") {
        debug!("{}", config);
    }
    if options.verbose {
        eprintln!("config: {:?}", config.path.as_ref().unwrap());
    }

    for k_eq_v in &options.variables {
        let name: String;
        let expr: String;
        let values: Vec<&str> = k_eq_v.splitn(2, "=").collect();
        if values.len() == 1 {
            name = values[0].into();
            expr = "".to_string();
        } else if values.len() == 2 {
            name = values[0].into();
            expr = values[1].into();
        } else {
            error!("unable to split '{}'", k_eq_v);
        }
        config.variables.insert(
            0, model::NamedVariable {
                name: name,
                expr: expr,
                value: None
            }
        );
    }

    config
}

/// Parse and apply configuration from a YAML/JSON string
pub fn parse(config_string: &str, verbose: bool,
             cfg: &mut model::Configuration) {

    config_yaml::parse(&config_string, verbose, cfg);
    // Initialize the configuration now that the values have been read.
    cfg.initialize();
}
