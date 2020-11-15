use yaml_rust::yaml::Yaml;
use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::YamlLoader;

use super::super::errors;
use super::super::model;
use super::super::path;


// Apply YAML Configuration from a string.
pub fn parse(
    string: &str,
    verbose: bool,
    config: &mut model::Configuration,
) -> Result<(), errors::GardenError> {

    let docs = YamlLoader::load_from_str(string).map_err(|scan_err| {
        errors::GardenError::ReadConfig { err: scan_err }
    })?;
    if docs.len() < 1 {
        return Err(errors::GardenError::EmptyConfiguration {
            path: config.get_path()?.into(),
        });
    }
    let doc = &docs[0];

    // Debug support
    if verbose {
        dump_node(doc, 1, "");
    }

    // garden.root
    if config.root.get_expr().is_empty() {
        if !get_str(&doc["garden"]["root"], config.root.get_expr_mut()) {
            // Default to the current directory when garden.root is unspecified
            // NOTE: this logic must be duplicated here for GARDEN_ROOT.
            // TODO: move GARDEN_ROOT initialization out of this so that
            // we can avoid this early initialization and do it in the outer
            // config::new() call.
            config.root.set_expr(path::current_dir_string());
        }

        if verbose {
            debug!("yaml: garden.root = {}", config.root.get_expr());
        }
    }

    // garden.shell
    if get_str(&doc["garden"]["shell"], &mut config.shell) && verbose {
        debug!("yaml: garden.shell = {}", config.shell);
    }

    // grafts
    if verbose {
        debug!("yaml: grafts");
    }
    if !get_grafts(&doc["grafts"], &mut config.grafts) && verbose {
        debug!("yaml: no grafts");
    }

    // variables
    if verbose {
        debug!("yaml: variables");
    }
    // Provide GARDEN_ROOT
    config.variables.push(
        model::NamedVariable::new(
            "GARDEN_ROOT".to_string(),
            config.root.get_expr().to_string(),
            None
        )
    );

    if let Some(config_path_raw) = config.dirname.as_ref() {
        // Calculate an absolute path for GARDEN_CONFIG_DIR.
        if let Ok(config_path) = config_path_raw.canonicalize() {
            config.variables.push(
                model::NamedVariable::new(
                    "GARDEN_CONFIG_DIR".to_string(),
                    config_path.to_string_lossy().to_string(),
                    None
                )
            );
        }
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
    if !get_trees(&doc["trees"], &doc["templates"], &mut config.trees) && verbose {
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

    Ok(())
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


/// Yaml -> String
fn get_str(yaml: &Yaml, string: &mut String) -> bool {
    if let Yaml::String(yaml_string) = yaml {
        *string = yaml_string.clone();
    }

    !string.is_empty()
}


/// Yaml::String or Yaml::Array<Yaml::String> -> Vec<String>
fn get_vec_str(yaml: &Yaml, vec: &mut Vec<String>) -> bool {

    if let Yaml::String(yaml_string) = yaml {
        vec.push(yaml_string.clone());
        return true;
    }

    if let Yaml::Array(ref yaml_vec) = yaml {
        for value in yaml_vec {
            if let Yaml::String(ref value_str) = value {
                vec.push(value_str.clone());
            }
        }
        return true;
    }
    return false;
}


/// Read NamedVariable definitions (variables)
fn get_variables(yaml: &Yaml, vec: &mut Vec<model::NamedVariable>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (k, v) in hash {
            let key = match k.as_str() {
                Some(key_value) => key_value.to_string(),
                None => continue,
            };
            match v {
                Yaml::String(ref yaml_str) => {
                    vec.push(model::NamedVariable::new(key, yaml_str.clone(), None));
                }
                Yaml::Array(ref yaml_array) => {
                    for value in yaml_array {
                        if let Yaml::String(ref yaml_str) = value {
                            vec.push(
                                model::NamedVariable::new(
                                    key.to_string(), yaml_str.clone(), None
                                )
                            );
                        }
                    }
                }
                Yaml::Integer(yaml_int) => {
                    let value = yaml_int.to_string();
                    vec.push(
                        model::NamedVariable::new(
                            key, value.clone(), Some(value.clone())
                        )
                    );
                }
                Yaml::Boolean(ref yaml_bool) => {
                    let value = bool_to_string(yaml_bool);
                    vec.push(
                        model::NamedVariable::new(
                            key, value.clone(), Some(value.clone())
                        )
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


fn bool_to_string(value: &bool) -> String {
    match *value {
        true => "true".into(),
        false => "false".into(),
    }
}

/// Read MultiVariable definitions (commands, environment)
fn get_multivariables(yaml: &Yaml, vec: &mut Vec<model::MultiVariable>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (k, v) in hash {
            let key = match k.as_str() {
                Some(key_value) => key_value.to_string(),
                None => continue,
            };
            match v {
                Yaml::String(ref yaml_str) => {
                    let variables = vec![
                        model::Variable::new(yaml_str.to_string(), None)
                    ];
                    vec.push(model::MultiVariable::new(key, variables));
                }
                Yaml::Array(ref yaml_array) => {
                    let mut variables = Vec::new();
                    for value in yaml_array {
                        if let Yaml::String(ref yaml_str) = value {
                            variables.push(
                                model::Variable::new(yaml_str.clone(), None)
                            );
                        }
                    }
                    vec.push(model::MultiVariable::new(key, variables));
                }
                Yaml::Integer(yaml_int) => {
                    let value = yaml_int.to_string();
                    let variables = vec![
                        model::Variable::new(value.clone(), Some(value)),
                    ];
                    vec.push(model::MultiVariable::new(key, variables));
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


/// Read template definitions
fn get_templates(yaml: &Yaml, templates: &mut Vec<model::Template>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (name, value) in hash {
            templates.push(get_template(name, value, yaml));
        }
        return true;
    }
    return false;
}


/// Read a single template definition
fn get_template(name: &Yaml, value: &Yaml, templates: &Yaml) -> model::Template {

    let mut template = model::Template::default();
    get_str(&name, &mut template.name);
    {
        let mut url = String::new();
        if get_str(&value["url"], &mut url) {
            template.remotes.push(
                model::NamedVariable::new("origin".to_string(), url, None)
            );
        }
    }

    // "environment" follow last-set-wins semantics.
    // Process the base templates in the specified order before processing
    // the template itself.
    get_vec_str(&value["extend"], &mut template.extend);
    for template_name in &template.extend {
        if let Yaml::Hash(_) = templates[template_name.as_ref()] {
            let mut base = get_template(
                &Yaml::String(template_name.clone()),
                &templates[template_name.as_ref()],
                templates,
            );

            template.environment.append(&mut base.environment);
            template.gitconfig.append(&mut base.gitconfig);
            // If multiple templates define "url" then the first one wins,
            // but only if we don't have url defined in the current template.
            if template.remotes.is_empty() {
                template.remotes.append(&mut base.remotes);
            }
        }
    }

    get_variables(&value["variables"], &mut template.variables);
    get_variables(&value["gitconfig"], &mut template.gitconfig);
    get_multivariables(&value["environment"], &mut template.environment);
    get_multivariables(&value["commands"], &mut template.commands);

    // These follow first-found semantics; process the base templates in
    // reverse order.
    for template_name in template.extend.iter().rev() {
        if let Yaml::Hash(_) = templates[template_name.as_ref()] {
            let mut base = get_template(
                &Yaml::String(template_name.clone()),
                &templates[template_name.as_ref()],
                templates,
            );

            template.variables.append(&mut base.variables);
            template.commands.append(&mut base.commands);
        }
    }

    return template;
}


/// Read tree definitions
fn get_trees(yaml: &Yaml, templates: &Yaml, trees: &mut Vec<model::Tree>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (name, value) in hash {
            if let Yaml::String(ref url) = value {
                trees.push(get_tree_from_url(name, url));
            } else {
                trees.push(get_tree(name, value, templates, hash, true));
            }
        }
        return true;
    }
    return false;
}



/// Return a tree from a simple "tree: <url>" entry
fn get_tree_from_url(name: &Yaml, url: &str) -> model::Tree {
    let mut tree = model::Tree::default();

    // Tree name
    get_str(&name, tree.get_name_mut());

    // Default to the name when "path" is unspecified.
    let tree_name = tree.get_name().to_string();
    tree.get_path_mut().set_expr(tree_name.to_string());
    tree.get_path().set_value(tree_name);

    // Register the ${TREE_NAME} variable.
    tree.variables.insert(
        0,
        model::NamedVariable::new(
            "TREE_NAME".to_string(), tree.get_name().clone(), None
        )
    );

    // Register the ${TREE_PATH} variable.
    tree.variables.insert(
        1,
        model::NamedVariable::new(
            "TREE_PATH".to_string(), tree.get_path().get_expr().clone(), None
        )
    );

    tree.remotes.push(
        model::NamedVariable::new("origin".to_string(), url.to_string(), None)
    );

    tree
}


/// Read a single tree definition
fn get_tree(
    name: &Yaml,
    value: &Yaml,
    templates: &Yaml,
    trees: &YamlHash,
    variables: bool,
) -> model::Tree {

    let mut tree = model::Tree::default();

    // Allow extending an existing tree by specifying "extend".
    let mut extend = String::new();
    if get_str(&value["extend"], &mut extend) {
        let tree_name = Yaml::String(extend);
        if let Some(ref tree_values) = trees.get(&tree_name) {
            tree = get_tree(&tree_name, tree_values, templates, trees, false);
            tree.remotes.truncate(1); // Keep origin only
            tree.templates.truncate(0); // Parent templates have already been processed.
        }
    }

    // Tree name
    get_str(&name, tree.get_name_mut());

    // Tree path
    if !get_str(&value["path"], tree.get_path_mut().get_expr_mut()) {
        // default to the name when "path" is unspecified
        let tree_name = tree.get_name().to_string();
        tree.get_path_mut().set_expr(tree_name.to_string());
        tree.get_path().set_value(tree_name);
    }

    // Add the TREE_NAME and TREE_PATH variables
    if variables {
        // Register the ${TREE_NAME} variable.
        tree.variables.insert(
            0,
            model::NamedVariable::new(
                "TREE_NAME".to_string(),
                tree.get_name().clone(),
                None
            )
        );
        // Register the ${TREE_PATH} variable.
        tree.variables.insert(
            1,
            model::NamedVariable::new(
                "TREE_PATH".to_string(),
                tree.get_path().get_expr().clone(),
                None
            )
        );
    }

    {
        let mut url = String::new();
        if get_str(&value["url"], &mut url) {
            tree.remotes.push(
                model::NamedVariable::new("origin".to_string(), url, None)
            );
        }
    }

    // Symlinks
    tree.is_symlink = get_str(&value["symlink"], tree.symlink.get_expr_mut());

    // Templates
    get_vec_str(&value["templates"], &mut tree.templates);

    // "environment" follow last-set-wins semantics.
    // Process the base templates in the specified order before processing
    // the template itself.
    for template_name in &tree.templates {
        if let Yaml::Hash(_) = templates[template_name.as_ref()] {
            let mut base = get_template(
                &Yaml::String(template_name.clone()),
                &templates[template_name.as_ref()],
                templates,
            );

            tree.environment.append(&mut base.environment);
            tree.gitconfig.append(&mut base.gitconfig);
            tree.commands.append(&mut base.commands);
            // If multiple templates define "url" then the first one wins,
            // but only if we don't have url defined in the current template.
            if tree.remotes.is_empty() {
                tree.remotes.append(&mut base.remotes);
            }
        }
    }

    get_variables(&value["variables"], &mut tree.variables);
    get_variables(&value["gitconfig"], &mut tree.gitconfig);
    get_multivariables(&value["environment"], &mut tree.environment);
    get_multivariables(&value["commands"], &mut tree.commands);

    // Remotes
    get_remotes(&value["remotes"], &mut tree.remotes);

    // These follow first-found semantics; process templates in
    // reverse order.
    for template_name in tree.templates.iter().rev() {
        if let Yaml::Hash(_) = templates[template_name.as_ref()] {
            let mut base = get_template(
                &Yaml::String(template_name.clone()),
                &templates[template_name.as_ref()],
                templates,
            );

            tree.variables.append(&mut base.variables);
        }
    }

    tree
}


/// Read Git remote repository definitions
fn get_remotes(yaml: &Yaml, remotes: &mut Vec<model::NamedVariable>) {
    if let Yaml::Hash(ref hash) = yaml {
        for (name, value) in hash {
            if let (Some(name_str), Some(value_str)) = (name.as_str(), value.as_str()) {
                remotes.push(
                    model::NamedVariable::new(
                        name_str.to_string(), value_str.to_string(), None
                    )
                );
            }
        }
    }
}


/// Read group definitions
fn get_groups(yaml: &Yaml, groups: &mut Vec<model::Group>) -> bool {
    if let Yaml::Hash(ref hash) = yaml {
        for (name, value) in hash {
            let mut group = model::Group::default();
            get_str(&name, group.get_name_mut());
            get_vec_str(&value, &mut group.members);
            groups.push(group);
        }
        return true;
    }
    return false;
}


/// Read garden definitions
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


/// Read a grafts: block into a Vec<Graft>.
fn get_grafts(yaml: &Yaml, grafts: &mut Vec<model::Graft>) -> bool {
    if let Yaml::Hash(ref yaml_hash) = yaml {
        for (name, value) in yaml_hash {
            let mut graft = get_graft(name, value);
            graft.index = grafts.len();
            grafts.push(graft);
        }
        true
    } else {
        false
    }
}


fn get_graft(name: &Yaml, graft: &Yaml) -> model::Graft {

    let mut graft_name = "".to_string();
    let mut config_expr = "".to_string();
    let mut root = "".to_string();
    let idx = 0;

    get_str(name, &mut graft_name);

    if !get_str(graft, &mut config_expr) {
        // The root was not specified.
        if let Yaml::Hash(ref _hash) = graft {
            // A config expression and root might be specified.
            get_str(&graft["config"], &mut config_expr);
            get_str(&graft["root"], &mut root);
        }
    }

    model::Graft {
        name: graft_name,
        root: root,
        config: None,
        config_expr: config_expr,
        index: idx,
    }
}


/// Read and parse YAML from a file path.
pub fn read_yaml<P>(path: P) -> Result<Yaml, errors::GardenError>
where
    P: std::convert::AsRef<std::path::Path> + std::fmt::Debug,
{

    let string = std::fs::read_to_string(&path).map_err(|io_err| {
        errors::GardenError::ReadFile {
            path: path.as_ref().into(),
            err: io_err,
        }
    })?;

    let mut docs = YamlLoader::load_from_str(&string).map_err(|scan_err| {
        errors::GardenError::ReadConfig { err: scan_err }
    })?;

    if docs.len() < 1 {
        return Err(errors::GardenError::EmptyConfiguration {
            path: path.as_ref().into(),
        });
    }

    add_missing_sections(&mut docs[0])?;

    Ok(docs[0].clone())
}


fn add_missing_sections(doc: &mut Yaml) -> Result<(), errors::GardenError> {
    // Garden core
    let mut good = doc["garden"].as_hash().is_some();
    if !good {
        if let Yaml::Hash(ref mut doc_hash) = doc {
            let key = Yaml::String("garden".into());
            doc_hash.insert(key, Yaml::Hash(YamlHash::new()));
        } else {
            return Err(errors::GardenError::InvalidConfiguration {
                msg: "document is not a hash".into(),
            });
        }
    }

    // Trees
    good = doc["trees"].as_hash().is_some();
    if !good {
        if let Yaml::Hash(ref mut doc_hash) = doc {
            let key = Yaml::String("trees".into());
            doc_hash.remove(&key);
            doc_hash.insert(key, Yaml::Hash(YamlHash::new()));
        } else {
            return Err(errors::GardenError::InvalidConfiguration {
                msg: "'trees' is not a hash".into(),
            });
        }
    }

    // Groups
    good = doc["groups"].as_hash().is_some();
    if !good {
        if let Yaml::Hash(ref mut doc_hash) = doc {
            let key = Yaml::String("groups".into());
            doc_hash.remove(&key);
            doc_hash.insert(key, Yaml::Hash(YamlHash::new()));
        } else {
            return Err(errors::GardenError::InvalidConfiguration {
                msg: "'groups' is not a hash".into(),
            });
        }
    }

    // Gardens
    good = doc["gardens"].as_hash().is_some();
    if !good {
        if let Yaml::Hash(ref mut doc_hash) = doc {
            let key = Yaml::String("gardens".into());
            doc_hash.remove(&key);
            doc_hash.insert(key, Yaml::Hash(YamlHash::new()));
        } else {
            return Err(errors::GardenError::InvalidConfiguration {
                msg: "'gardens' is not a hash".into(),
            });
        }
    }

    Ok(())
}


pub fn empty_doc() -> Yaml {
    let mut doc = Yaml::Hash(YamlHash::new());
    add_missing_sections(&mut doc).ok();

    doc
}
