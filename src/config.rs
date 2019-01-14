extern crate dirs;
extern crate yaml_rust;

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
    pub variables: Vec<model::Variable>,
    pub shell: std::path::PathBuf,
    pub environment: Vec<model::NameValue>,
    pub commands: Vec<model::NameValue>,
    pub tree_search_path: Vec<std::path::PathBuf>,
    pub root_path: std::path::PathBuf,
    pub gardens: Vec<model::Garden>,
    pub groups: Vec<String>,
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
    let variables = Vec::new();
    let environment = Vec::new();
    let commands = Vec::new();
    let gardens = Vec::new();
    let groups = Vec::new();
    let tree_search_path = Vec::new();
    let root_path = std::path::PathBuf::new();

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
        debug!("config path is {:?} {}", path,
               match found {
                   true => "",
                   false => " (NOT FOUND)",
               });
    }

    let config = Configuration {
        path: path,
        file_format: file_format,
        shell: shell,
        variables: variables,
        environment: environment,
        commands: commands,
        gardens: gardens,
        groups: groups,
        root_path: root_path,
        tree_search_path: tree_search_path,
        verbose: verbose,
    };

    if found {
        // parse yaml
        match file_format {
            FileFormat::YAML => {
                if verbose {
                    debug!("file format: yaml");
                }
                read_yaml_config(&config, verbose);
            }
            FileFormat::JSON => {
                if verbose {
                    debug!("file format: json");
                }
            }
            _ => {
                error!("unsupported config file format");
            }
        }
    }
    return config;
}


fn print_indent(indent: usize) {
    for _ in 0..indent {
        print!("    ");
    }
}


fn dump_node(doc: &yaml_rust::yaml::Yaml, indent: usize) {
    match *doc {
        yaml_rust::yaml::Yaml::Array(ref v) => {
            for x in v {
                dump_node(x, indent + 1);
            }
        }
        yaml_rust::yaml::Yaml::Hash(ref h) => {
            for (k, v) in h {
                print_indent(indent);
                println!("{:?}:", k);
                dump_node(v, indent + 1);
            }
        }
        _ => {
            print_indent(indent);
            println!("{:?}", doc);
        }
    }
}


fn read_yaml_config(mut config: &Configuration, verbose: bool) {
    let config_string = unwrap_or_err!(
        std::fs::read_to_string(&config.path),
        "unable to read {:?}: {}", config.path);

    let docs = unwrap_or_err!(
        yaml_rust::YamlLoader::load_from_str(&config_string),
        "{:?}: {}", config.path);

    read_yaml_docs(config, &docs);
}

fn read_yaml_docs(mut config: &Configuration, docs: &Vec<yaml_rust::Yaml>) {
    if docs.len() < 1 {
        error!("empty yaml configuration: {:?}", config.path);
    }

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    // Debug support
    //println!("{:?}", doc);
    dump_node(doc, 0);
}
