extern crate dirs;
extern crate yaml_rust;

use super::cmd::{debug,error};
use super::model;

#[derive(Clone, Copy)]
pub enum FileFormat {
    JSON,
    YAML,
    UNKNOWN,
}

// Configuration contains multiple gardens
pub struct Configuration {
    pub path: std::path::PathBuf,
    pub file_format: FileFormat,
    pub shell: std::path::PathBuf,
    pub environ: Vec<model::NameValue>,
    pub tree_search_path: Vec<std::path::PathBuf>,
    pub tree_path: std::path::PathBuf,
    pub gardens: Vec<model::Garden>,
    pub verbose: bool,
}

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

    // . Current directory
    paths.push(std::env::current_dir().unwrap());

    // ./garden
    let mut current_garden_dir  = std::env::current_dir().unwrap();
    current_garden_dir.push("garden");
    if current_garden_dir.exists() {
        paths.push(current_garden_dir);
    }

    // ./etc/garden
    let mut current_etc_garden_dir  = std::env::current_dir().unwrap();
    current_etc_garden_dir.push("etc");
    current_etc_garden_dir.push("garden");
    if current_etc_garden_dir.exists() {
        paths.push(current_etc_garden_dir);
    }

    // ~/.config/garden
    let mut home_config_dir = dirs::home_dir().unwrap();
    home_config_dir.push(".config");
    home_config_dir.push("garden");
    if home_config_dir.exists() {
        paths.push(home_config_dir);
    }

    // ~/etc/garden
    let mut home_etc_dir = dirs::home_dir().unwrap();
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

pub fn new(verbose: bool) -> Configuration {
    let mut file_format = FileFormat::UNKNOWN;
    let mut path = std::path::PathBuf::new();
    let shell = std::path::PathBuf::new();
    let search_path = search_path();
    let environ = vec!();
    let gardens = vec!();
    let tree_search_path = vec!();
    let tree_path = std::path::PathBuf::new();

    // Find garden.yaml in the search path
    let mut found = false;
    for entry in &search_path {
        let mut yaml_candidate = entry.to_path_buf();
        yaml_candidate.push("garden.yaml");
        if yaml_candidate.exists() {
            path = yaml_candidate.to_path_buf();
            file_format = FileFormat::YAML;
            found = true;
            break;
        }

        let mut json_candidate  = entry.to_path_buf();
        json_candidate.push("garden.json");
        if json_candidate.exists() {
            path = json_candidate.to_path_buf();
            file_format = FileFormat::JSON;
            found = true;
            break;
        }
    }

    if verbose {
        debug(format_args!("config path is {} {}",
                           path.to_string_lossy(),
                           match found {
                               true => "",
                               false => " (NOT FOUND)",
                           }));
    }

    let config = Configuration {
        path: path,
        file_format: file_format,
        environ: environ,
        shell: shell,
        gardens: gardens,
        tree_path: tree_path,
        tree_search_path: tree_search_path,
        verbose: verbose,
    };

    if found {
        // parse yaml
        match file_format {
            FileFormat::YAML => {
                if verbose {
                    debug(format_args!("parse yaml"));
                }
                read_yaml_config(&config, verbose);
            }
            FileFormat::JSON => {
                if verbose {
                    debug(format_args!("parse json"));
                }
            }
            _ => {
                error(format_args!("unsupported config file format"));
            }
        }
    }
    return config;
}

fn read_yaml_config(mut config: &Configuration, verbose: bool) {
}
