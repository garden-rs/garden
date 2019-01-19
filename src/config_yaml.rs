extern crate yaml_rust;

use super::config;

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


pub fn read(mut config: &config::Configuration, verbose: bool) {
    let config_string = unwrap_or_err!(
        std::fs::read_to_string(&config.path),
        "unable to read {:?}: {}", config.path);

    let docs = unwrap_or_err!(
        yaml_rust::YamlLoader::load_from_str(&config_string),
        "{:?}: {}", config.path);

    if docs.len() < 1 {
        error!("empty yaml configuration: {:?}", config.path);
    }

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    // Debug support
    if verbose {
        dump_node(doc, 1);
    }

    // Evaluate garden.root
}
