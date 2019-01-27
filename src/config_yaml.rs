use std::string::ToString;

extern crate yaml_rust;
use self::yaml_rust::yaml::Yaml;
use self::yaml_rust::YamlLoader;

use super::model;


// Apply YAML Configuration from a string.
pub fn parse(string: &String, verbose: bool,
             config: &mut model::Configuration) {

    let docs = unwrap_or_err!(
        YamlLoader::load_from_str(string.as_ref()),
        "{:?}: {}", config.path);

    if docs.len() < 1 {
        error!("empty yaml configuration: {:?}", config.path);
    }

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    // Debug support
    if verbose {
        dump_node(doc, 1, "");
    }

    // garden.environment_variables
    if get_bool(&doc["garden"]["environment_variables"],
                &mut config.environment_variables) && verbose {
        debug!("yaml: garden.environment_variables = {}",
               config.environment_variables);
    }

    // garden.root
    if get_path(&doc["garden"]["root"], &mut config.root_path) && verbose {
        debug!("yaml: garden.root = {:?}", config.root_path);
    }

    // garden.shell
    if get_path(&doc["garden"]["shell"], &mut config.root_path) {
        debug!("yaml: garden.shell = {}", config.shell.to_str().unwrap());
    }

    // variables
    if verbose {
        debug!("yaml: read variables");
    }
    if !get_variables(&doc["variables"], &mut config.variables) && verbose {
        debug!("yaml: no variables");
    }

    // commands
    if verbose {
        debug!("yaml: read command multivariables");
    }
    if !get_multivariables(&doc["commands"], &mut config.commands) && verbose {
        debug!("yaml: no commands");
    }
}


fn print_indent(indent: usize) {
    for _ in 0..indent {
        print!("    ");
    }
}


fn dump_node(doc: &Yaml, indent: usize, prefix: &str) {
    match *doc {
        Yaml::String(ref s) => {
            print_indent(indent);
            println!("{}\"{}\"", prefix, s);
        }
        Yaml::Array(ref v) => {
            for x in v {
                dump_node(x, indent + 1, "- ");
            }
        }
        Yaml::Hash(ref hash) => {
            for (k, v) in hash {
                print_indent(indent);
                match k {
                    Yaml::String(ref x) => {
                        println!("{}{}:", prefix, x);
                    }
                    _ => {
                        println!("{}{:?}:", prefix, k);
                    }
                }
                dump_node(v, indent + 1, prefix);
            }
        }
        _ => {
            print_indent(indent);
            println!("{:?}", doc);
        }
    }
}


fn get_bool(yaml: &Yaml, value: &mut bool) -> bool {
    if let Yaml::Boolean(boolean) = yaml {
        *value = *boolean;
        return true;
    }
    return false;
}


fn get_str(yaml: &Yaml, string: &mut String) -> bool {
    if let Yaml::String(yaml_string) = yaml {
        *string = yaml_string.to_string();
        return true;
    }
    return false;
}


fn get_path(yaml: &Yaml, pathbuf: &mut std::path::PathBuf) -> bool {
    if let Yaml::String(yaml_string) = yaml {
        *pathbuf = std::path::PathBuf::from(yaml_string.to_string());
        return true;
    }
    return false;
}


fn get_vec_str(yaml: &Yaml, vec: &mut Vec<String>) -> bool {

    if let Yaml::String(yaml_string) = yaml {
        vec.push(yaml_string.to_string());
        return true;
    }

    if let Yaml::Array(ref yaml_vec) = yaml {
        for value in yaml_vec {
            if let Yaml::String(ref value_str) = value {
                vec.push(value_str.to_string());
            }
        }
        return true;
    }
    return false;
}


fn get_variables(yaml: &Yaml, vec: &mut Vec<model::NamedVariable>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (k, v) in hash {
            match v {
                Yaml::String(ref yaml_str) => {
                    vec.push(
                        model::NamedVariable{
                            name: k.as_str().unwrap().to_string(),
                            var: model::Variable{
                                expr: yaml_str.to_string(),
                                value: None,
                            },
                        });
                }
                Yaml::Array(ref yaml_array) => {
                    for value in yaml_array {
                        if let Yaml::String(ref yaml_str) = value {
                            vec.push(
                                model::NamedVariable{
                                    name: k.as_str().unwrap().to_string(),
                                    var: model::Variable{
                                        expr: yaml_str.to_string(),
                                        value: None,
                                    },
                                }
                            );
                        }
                    }
                }
                _ => {
                    dump_node(yaml, 0, "");
                    error!("invalid variables");
                }
            }
        }
        return true;
    }
    return false;
}


fn get_multivariables(yaml: &Yaml,
                      vec: &mut Vec<model::MultiVariable>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (k, v) in hash {
            match v {
                Yaml::String(ref yaml_str) => {
                    vec.push(
                        model::MultiVariable{
                            name: k.as_str().unwrap().to_string(),
                            values: vec!(
                                model::Variable{
                                    expr: yaml_str.to_string(),
                                    value: None,
                                }
                            )
                        }
                    );
                }
                Yaml::Array(ref yaml_array) => {
                    let mut values = Vec::new();
                    for value in yaml_array {
                        if let Yaml::String(ref yaml_str) = value {
                            values.push(
                                model::Variable{
                                    expr: yaml_str.to_string(),
                                    value: None,
                                }
                            );
                        }
                    }
                    vec.push(
                        model::MultiVariable{
                            name: k.as_str().unwrap().to_string(),
                            values: values,
                        }
                    );
                }
                _ => {
                    dump_node(yaml, 0, "");
                    error!("invalid variables");
                }
            }
        }
        return true;
    }
    return false;
}
