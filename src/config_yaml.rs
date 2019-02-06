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
    if get_path(&doc["garden"]["shell"], &mut config.root_path) && verbose {
        debug!("yaml: garden.shell = {}", config.shell.to_str().unwrap());
    }

    // variables
    if verbose {
        debug!("yaml: variables");
    }
    if !get_variables(&doc["variables"], &mut config.variables) && verbose {
        debug!("yaml: no variables");
    }

    // commands
    if verbose {
        debug!("yaml: commands");
    }
    if !get_multivariables(&doc["commands"], &mut config.commands) && verbose {
        debug!("yaml: no commands");
    }

    // templates
    if verbose {
        debug!("yaml: templates");
    }
    if !get_templates(&doc["templates"], &mut config.templates) && verbose {
        debug!("yaml: no templates");
    }

    // trees
    if verbose {
        debug!("yaml: trees");
    }
    if !get_trees(&doc["trees"], &mut config.trees) && verbose {
        debug!("yaml: no trees");
    }

    // groups
    if verbose {
        debug!("yaml: groups");
    }
    if !get_groups(&doc["groups"], &mut config.groups) && verbose {
        debug!("yaml: no groups");
    }

    // gardens
    if verbose {
        debug!("yaml: gardens");
    }
    if !get_gardens(&doc["gardens"], &mut config.gardens) && verbose {
        debug!("yaml: no gardens");
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
                                model::NamedVariable {
                                    name: k.as_str().unwrap().to_string(),
                                    var: model::Variable {
                                        expr: yaml_str.to_string(),
                                        value: None,
                                    },
                                }
                            );
                        }
                    }
                }
                Yaml::Integer(yaml_int) => {
                    vec.push(
                        model::NamedVariable {
                            name: k.as_str().unwrap().to_string(),
                            var: model::Variable {
                                expr: yaml_int.to_string(),
                                value: Some(yaml_int.to_string()),
                            }
                        }
                    );
                }
                _ => {
                    dump_node(v, 1, "");
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
                Yaml::Integer(yaml_int) => {
                    vec.push(
                        model::MultiVariable {
                            name: k.as_str().unwrap().to_string(),
                            values: vec!(
                                model::Variable {
                                    expr: yaml_int.to_string(),
                                    value: Some(yaml_int.to_string()),
                                }
                            ),
                        }
                    );
                }
                _ => {
                    dump_node(v, 1, "");
                    error!("invalid variables");
                }
            }
        }
        return true;
    }
    return false;
}


fn get_templates(yaml: &Yaml, templates: &mut Vec<model::Template>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (name, value) in hash {
            templates.push(get_template(name, value));
        }
        return true;
    }
    return false;
}


fn get_template(name: &Yaml, value: &Yaml) -> model::Template {
    let mut template = model::Template::default();
    get_str(&name, &mut template.name);
    get_vec_str(&value["extend"], &mut template.extend);
    get_variables(&value["variables"], &mut template.variables);
    get_variables(&value["gitconfig"], &mut template.gitconfig);
    get_multivariables(&value["environment"], &mut template.environment);
    get_multivariables(&value["commands"], &mut template.commands);

    return template;
}


fn get_trees(yaml: &Yaml, trees: &mut Vec<model::Tree>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (name, value) in hash {
            trees.push(get_tree(name, value));
        }
        return true;
    }
    return false;
}


fn get_tree(name: &Yaml, value: &Yaml) -> model::Tree {
    let mut tree = model::Tree::default();
    get_str(&name, &mut tree.name);
    get_str(&value["path"], &mut tree.path);
    {
        let mut url = String::new();
        if get_str(&value["url"], &mut url) {
            tree.remotes.push(
                model::Remote {
                    name: "origin".to_string(),
                    url: url.to_string(),
                });
        }
    }
    get_vec_str(&value["templates"], &mut tree.templates);
    get_variables(&value["variables"], &mut tree.variables);
    get_variables(&value["gitconfig"], &mut tree.gitconfig);
    get_multivariables(&value["environment"], &mut tree.environment);
    get_multivariables(&value["commands"], &mut tree.commands);

    return tree;
}


fn get_groups(yaml: &Yaml, groups: &mut Vec<model::Group>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (name, value) in hash {
            let mut group = model::Group::default();
            get_str(&name, &mut group.name);
            get_vec_str(&value, &mut group.members);
            groups.push(group);
        }
        return true;
    }
    return false;
}


fn get_gardens(yaml: &Yaml, gardens: &mut Vec<model::Garden>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (name, value) in hash {
            let mut garden = model::Garden::default();
            get_str(&name, &mut garden.name);
            get_vec_str(&value["groups"], &mut garden.groups);
            get_vec_str(&value["trees"], &mut garden.trees);
            get_variables(&value["variables"], &mut garden.variables);
            get_multivariables(&value["environment"], &mut garden.environment);
            get_multivariables(&value["commands"], &mut garden.commands);
            get_variables(&value["gitconfig"], &mut garden.gitconfig);
            gardens.push(garden);
        }
        return true;
    }
    return false;
}
