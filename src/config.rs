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

    return paths;
}


pub fn new(config: &Option<std::path::PathBuf>, verbose: bool)
-> model::Configuration {

    let mut cfg = model::Configuration::new();
    cfg.verbose = verbose;

    // Find garden.yaml in the search path
    let mut found = false;
    if let Some(config_path) = config {
        if config_path.is_file() {
            cfg.path = Some(config_path.to_path_buf());
            found = true;
        }
    }

    if !found {
        for entry in search_path() {
            let formats = vec!("yaml", "json");
            for fmt in &formats {
                let mut candidate = entry.to_path_buf();
                candidate.push(String::from("garden.") + fmt);
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
            cfg, "unable to read {:?}: {}", cfg.path);
        parse(&config_string, verbose, &mut cfg);
    }

    return cfg;
}


/// Parse and apply configuration from a YAML/JSON string
pub fn parse(config_string: &String,
             verbose: bool, mut cfg: &mut model::Configuration) {

    config_yaml::parse(&config_string, verbose, &mut cfg);
    // Initialize the configuration now that the values have been read.
    cfg.initialize();
}
