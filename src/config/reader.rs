use super::super::errors;
use super::super::eval;
use super::super::model;
use super::super::path;
use super::super::syntax;

use indexmap::{IndexMap, IndexSet};
use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::yaml::Yaml;
use yaml_rust::YamlLoader;

use std::collections::HashMap;

// Apply YAML Configuration from a string.
pub fn parse(
    app_context: &model::ApplicationContext,
    string: &str,
    config_verbose: u8,
    config: &mut model::Configuration,
) -> Result<(), errors::GardenError> {
    parse_recursive(app_context, string, config_verbose, config, None, true)
}

fn parse_recursive(
    app_context: &model::ApplicationContext,
    string: &str,
    config_verbose: u8,
    config: &mut model::Configuration,
    current_include: Option<&std::path::Path>,
    is_root_config: bool,
) -> Result<(), errors::GardenError> {
    let docs =
        YamlLoader::load_from_str(string).map_err(|scan_err| errors::GardenError::ReadConfig {
            err: scan_err,
            path: config.get_path_for_display(),
        })?;
    if docs.is_empty() {
        return Err(errors::GardenError::EmptyConfiguration {
            path: config.get_path()?.into(),
        });
    }
    let doc = &docs[0];

    // Debug support
    if config_verbose > 2 {
        dump_node(doc, 1, "");
    }

    // garden.root
    if config.root.is_empty() {
        if !get_str(&doc["garden"]["root"], config.root.get_expr_mut()) {
            // Default to the current directory when garden.root is unspecified
            // NOTE: this logic must be duplicated here for GARDEN_ROOT.
            // TODO: move GARDEN_ROOT initialization out of this so that
            // we can avoid this early initialization and do it in the outer
            // config::new() call.
            config.root.set_expr(path::current_dir_string());
        }

        if config_verbose > 0 {
            debug!("yaml: garden.root = {}", config.root.get_expr());
        }
    }

    // garden.shell
    if get_str(&doc["garden"]["shell"], &mut config.shell) && config_verbose > 0 {
        debug!("yaml: garden.shell = {}", config.shell);
    }
    // garden.tree-branches
    if get_bool(&doc["garden"]["tree-branches"], &mut config.tree_branches) && config_verbose > 0 {
        debug!("yaml: garden.tree-branches = {}", config.tree_branches);
    }

    // GARDEN_ROOT and GARDEN_CONFIG_DIR are relative to the root configuration.
    // Referencing these variables from garden files included using garden.includes
    // resolves to the root config's location, not the included location.
    if config_verbose > 1 {
        debug!("yaml: built-in variables");
    }
    if is_root_config {
        // Provide GARDEN_ROOT.
        config.variables.insert(
            string!("GARDEN_ROOT"),
            model::Variable::new(config.root.get_expr().to_string(), None),
        );

        if let Some(config_path_raw) = config.dirname.as_ref() {
            // Calculate an absolute path for GARDEN_CONFIG_DIR.
            if let Ok(config_path) = config_path_raw.canonicalize() {
                config.variables.insert(
                    string!("GARDEN_CONFIG_DIR"),
                    model::Variable::new(config_path.to_string_lossy().to_string(), None),
                );
            }
        }
    }

    // Variables are read early to make them available to config.eval_config_pathbuf_from_include().
    // Variables are reloaded after "includes" to give the current garden file the highest priority.
    if !get_variables_hashmap(&doc["variables"], &mut config.variables) && config_verbose > 1 {
        debug!("yaml: no variables");
    }

    // Process "includes" after initializing the GARDEN_ROOT and GARDEN_CONFIG_DIR.
    // This allows the path strings to reference these ${variables}.
    // This also means that variables defined by the outer-most garden config
    // override the same variables when also defined in an included garden file.
    let mut config_includes = Vec::new();
    if get_vec_variables(&doc["garden"]["includes"], &mut config_includes) {
        for garden_include in &config_includes {
            let pathbuf = match config.eval_config_pathbuf_from_include(
                app_context,
                current_include,
                garden_include.get_expr(),
            ) {
                Some(pathbuf) => pathbuf,
                None => continue,
            };
            if !pathbuf.exists() {
                if config_verbose > 0 {
                    debug!(
                        "warning: garden.includes entry not found: {:?} -> {:?}",
                        garden_include, pathbuf
                    );
                }
                continue;
            }
            if pathbuf.exists() {
                if let Ok(content) = std::fs::read_to_string(&pathbuf) {
                    parse_recursive(
                        app_context,
                        &content,
                        config_verbose,
                        config,
                        Some(&pathbuf),
                        false,
                    )
                    .unwrap_or(());
                }
            }
        }

        // Reload variables after processing includes. This gives the local garden file the highest priority
        // when defining variables while also making variables available to the "includes" lines.
        if !get_variables_hashmap(&doc["variables"], &mut config.variables) && config_verbose > 1 {
            debug!("yaml: no reloaded variables");
        }
    }

    // grafts
    if config_verbose > 1 {
        debug!("yaml: grafts");
    }
    if !get_grafts(&doc["grafts"], &mut config.grafts) && config_verbose > 1 {
        debug!("yaml: no grafts");
    }

    get_multivariables(&doc["environment"], &mut config.environment);

    // commands
    if config_verbose > 1 {
        debug!("yaml: commands");
    }
    if !get_multivariables_hashmap(&doc["commands"], &mut config.commands) && config_verbose > 1 {
        debug!("yaml: no commands");
    }

    // templates
    if config_verbose > 1 {
        debug!("yaml: templates");
    }
    if !get_templates(
        &doc["templates"],
        &config.templates.clone(),
        &mut config.templates,
    ) && config_verbose > 1
    {
        debug!("yaml: no templates");
    }

    // trees
    if config_verbose > 1 {
        debug!("yaml: trees");
    }
    if !get_trees(app_context, config, &doc["trees"]) && config_verbose > 1 {
        debug!("yaml: no trees");
    }

    // groups
    if config_verbose > 1 {
        debug!("yaml: groups");
    }
    if !get_groups(&doc["groups"], &mut config.groups) && config_verbose > 1 {
        debug!("yaml: no groups");
    }

    // gardens
    if config_verbose > 1 {
        debug!("yaml: gardens");
    }
    if !get_gardens(&doc["gardens"], &mut config.gardens) && config_verbose > 1 {
        debug!("yaml: no gardens");
    }

    Ok(())
}

fn print_indent(indent: usize) {
    for _ in 0..indent {
        print!("    ");
    }
}

fn dump_node(yaml: &Yaml, indent: usize, prefix: &str) {
    match yaml {
        Yaml::String(value) => {
            print_indent(indent);
            println!("{prefix}\"{value}\"");
        }
        Yaml::Array(value) => {
            for x in value {
                dump_node(x, indent + 1, "- ");
            }
        }
        Yaml::Hash(hash) => {
            for (k, v) in hash {
                print_indent(indent);
                match k {
                    Yaml::String(x) => {
                        println!("{prefix}{x}:");
                    }
                    _ => {
                        println!("{prefix}{k:?}:");
                    }
                }
                dump_node(v, indent + 1, prefix);
            }
        }
        _ => {
            print_indent(indent);
            println!("{yaml:?}");
        }
    }
}

/// Yaml -> String
fn get_str(yaml: &Yaml, string: &mut String) -> bool {
    match yaml {
        Yaml::String(yaml_string) => {
            *string = yaml_string.clone();
            !string.is_empty()
        }
        _ => false,
    }
}

/// Yaml -> i64
fn get_i64(yaml: &Yaml, value: &mut i64) -> bool {
    match yaml {
        Yaml::Integer(yaml_integer) => {
            *value = *yaml_integer;
            true
        }
        _ => false,
    }
}

/// Yaml -> bool
fn get_bool(yaml: &Yaml, value: &mut bool) -> bool {
    match yaml {
        Yaml::Boolean(yaml_bool) => {
            *value = *yaml_bool;
            true
        }
        _ => false,
    }
}

/// Yaml::String or Yaml::Array<Yaml::String> -> Vec<String>
fn get_indexset_str(yaml: &Yaml, values: &mut IndexSet<String>) -> bool {
    match yaml {
        Yaml::String(yaml_string) => {
            values.insert(yaml_string.clone());
            true
        }
        Yaml::Array(yaml_vec) => {
            for value in yaml_vec {
                if let Yaml::String(value_str) = value {
                    values.insert(value_str.clone());
                }
            }
            true
        }
        _ => false,
    }
}

/// Yaml::String or Yaml::Array<Yaml::String> -> Vec<Variable>
fn get_vec_variables(yaml: &Yaml, vec: &mut Vec<model::Variable>) -> bool {
    match yaml {
        Yaml::String(yaml_string) => {
            vec.push(model::Variable::new(yaml_string.clone(), None));
            true
        }
        Yaml::Array(yaml_vec) => {
            for value in yaml_vec {
                if let Yaml::String(value_str) = value {
                    vec.push(model::Variable::new(value_str.clone(), None));
                }
            }
            true
        }
        _ => false,
    }
}

// Yaml::String -> Variable
fn get_variable(yaml: &Yaml, value: &mut model::Variable) -> bool {
    match yaml {
        Yaml::String(yaml_string) => {
            value.set_expr(yaml_string.to_string());
            true
        }
        _ => false,
    }
}

/// Read variable definitions from a yaml::HashMap into a VariablesHashMap
fn get_variables_hashmap(yaml: &Yaml, hashmap: &mut model::VariableHashMap) -> bool {
    match yaml {
        Yaml::Hash(hash) => {
            for (k, v) in hash {
                let key = match k.as_str() {
                    Some(key_value) => key_value.to_string(),
                    None => {
                        continue;
                    }
                };
                match v {
                    Yaml::String(yaml_str) => {
                        hashmap.insert(key, model::Variable::new(yaml_str.clone(), None));
                    }
                    Yaml::Array(yaml_array) => {
                        for value in yaml_array {
                            if let Yaml::String(yaml_str) = value {
                                hashmap.insert(
                                    key.to_owned(),
                                    model::Variable::new(
                                        yaml_str.clone(),
                                        None, // Defer resolution of string values.
                                    ),
                                );
                            }
                        }
                    }
                    Yaml::Integer(yaml_int) => {
                        let value = yaml_int.to_string();
                        hashmap.insert(
                            key,
                            model::Variable::new(
                                value.clone(),
                                Some(value.clone()), // Integer values are already resolved.
                            ),
                        );
                    }
                    Yaml::Boolean(yaml_bool) => {
                        let value = syntax::bool_to_string(*yaml_bool);
                        hashmap.insert(
                            key,
                            model::Variable::new(
                                value.clone(),
                                Some(value.clone()), // Booleans are already resolved.
                            ),
                        );
                    }
                    _ => {
                        dump_node(v, 1, "");
                        error!("invalid variables");
                    }
                }
            }
            true
        }
        _ => false,
    }
}

/// Read MultiVariable definitions (commands, environment)
fn get_multivariables(yaml: &Yaml, vec: &mut Vec<model::MultiVariable>) -> bool {
    if let Yaml::Hash(hash) = yaml {
        for (k, v) in hash {
            let key = match k.as_str() {
                Some(key_value) => key_value.to_string(),
                None => continue,
            };
            match v {
                Yaml::String(yaml_str) => {
                    let variables = vec![model::Variable::new(yaml_str.to_string(), None)];
                    vec.push(model::MultiVariable::new(key, variables));
                }
                Yaml::Array(yaml_array) => {
                    let mut variables = Vec::new();
                    for value in yaml_array {
                        if let Yaml::String(yaml_str) = value {
                            variables.push(model::Variable::new(yaml_str.clone(), None));
                        }
                    }
                    vec.push(model::MultiVariable::new(key, variables));
                }
                Yaml::Integer(yaml_int) => {
                    let value = yaml_int.to_string();
                    let variables = vec![model::Variable::new(value.clone(), Some(value))];
                    vec.push(model::MultiVariable::new(key, variables));
                }
                _ => {
                    dump_node(v, 1, "");
                    error!("invalid configuration");
                }
            }
        }

        return true;
    }

    false
}

/// Read a mapping of String to Vec of Variables into a MultiVariableHashMap
fn get_multivariables_hashmap(
    yaml: &Yaml,
    multivariables: &mut model::MultiVariableHashMap,
) -> bool {
    match yaml {
        Yaml::Hash(hash) => {
            for (k, v) in hash {
                let key = match k.as_str() {
                    Some(key_value) => key_value.to_string(),
                    None => continue,
                };
                match v {
                    Yaml::String(yaml_str) => {
                        let variables = vec![model::Variable::new(yaml_str.to_string(), None)];
                        multivariables.insert(key, variables);
                    }
                    Yaml::Array(yaml_array) => {
                        let mut variables = Vec::new();
                        for value in yaml_array {
                            if let Yaml::String(yaml_str) = value {
                                variables.push(model::Variable::new(yaml_str.clone(), None));
                            }
                        }
                        multivariables.insert(key, variables);
                    }
                    Yaml::Integer(yaml_int) => {
                        // Ints are already resolved.
                        let value = yaml_int.to_string();
                        let variables = vec![model::Variable::new(value.clone(), Some(value))];
                        multivariables.insert(key, variables);
                    }
                    Yaml::Boolean(yaml_bool) => {
                        // Booleans are already resolved.
                        let value = syntax::bool_to_string(*yaml_bool);
                        let variables = vec![model::Variable::new(value.clone(), Some(value))];
                        multivariables.insert(key, variables);
                    }
                    _ => {
                        dump_node(v, 1, "");
                        error!("invalid variables");
                    }
                }
            }

            true
        }
        _ => false,
    }
}

/// Read template definitions
fn get_templates(
    yaml: &Yaml,
    config_templates: &HashMap<String, model::Template>,
    templates: &mut HashMap<String, model::Template>,
) -> bool {
    match yaml {
        Yaml::Hash(hash) => {
            for (name, value) in hash {
                let template_name = match &name.as_str() {
                    Some(template_name) => template_name.to_string(),
                    None => continue,
                };
                templates.insert(
                    template_name,
                    get_template(name, value, config_templates, yaml),
                );
            }
            true
        }
        _ => false,
    }
}

/// Read a single template definition
fn get_template(
    name: &Yaml,
    value: &Yaml,
    config_templates: &HashMap<String, model::Template>,
    templates: &Yaml,
) -> model::Template {
    let mut template = model::Template::default();
    get_str(name, template.get_name_mut());

    {
        let mut url = String::new();
        // If the YAML configuration is just a single string value then the template
        // expands out to url: <string-value> only.
        // templates:
        //   example: git://git.example.org/example/repo.git
        if get_str(value, &mut url) {
            template
                .tree
                .remotes
                .insert(string!("origin"), model::Variable::new(url, None));
            return template;
        }
        // If a <url> is configured then populate the "origin" remote.
        // The first remote is "origin" by convention.
        if get_str(&value["url"], &mut url) {
            template
                .tree
                .remotes
                .insert(string!("origin"), model::Variable::new(url, None));
        }
    }

    // Process the base templates in the specified order before processing
    // the template itself. Any "VAR=" variables will be overridden
    // by the tree entry itself, or the last template processed.
    // "environment" follow last-set-wins semantics.
    get_indexset_str(&value["extend"], &mut template.extend);
    for template_name in &template.extend {
        // First check if we have this template in the local YAML data.
        // We check here first so that parsing is not order-dependent.
        if let Yaml::Hash(_) = templates[template_name.as_ref()] {
            let base = get_template(
                &Yaml::String(template_name.clone()),
                &templates[template_name.as_ref()],
                config_templates,
                templates,
            );

            base.apply(&mut template.tree);
        } else {
            // If the template didn't exist in the local YAML then read it from
            // the previously-parsed templates. This allows templates to be used
            // from include files where the template definition is in a different
            // file and not present in the current YAML payload.
            if let Some(base) = config_templates.get(template_name) {
                base.apply(&mut template.tree);
            }
        }
        // The base templates were already processed.
        template.tree.templates.truncate(0);
    }

    get_tree_fields(value, &mut template.tree);

    template
}

/// Read tree definitions
fn get_trees(
    app_context: &model::ApplicationContext,
    config: &mut model::Configuration,
    yaml: &Yaml,
) -> bool {
    match yaml {
        Yaml::Hash(hash) => {
            for (name, value) in hash {
                if let Yaml::String(url) = value {
                    // If the tree already exists then update it, otherwise create a new entry.
                    let tree = get_tree_from_url(name, url);
                    if let Some(current_tree) = config.trees.get_mut(tree.get_name()) {
                        current_tree.clone_from_tree(&tree);
                    } else {
                        config.trees.insert(tree.get_name().to_string(), tree);
                    }
                } else {
                    let tree = get_tree(app_context, config, name, value, hash, true);

                    // Should we replace the current entry or sparsely override it?
                    // We sparsely override by default.
                    let replace = match value["replace"] {
                        Yaml::Boolean(value) => value,
                        _ => false,
                    };

                    let current_tree_opt = config.trees.get_mut(tree.get_name());
                    match current_tree_opt {
                        Some(current_tree) if !replace => {
                            current_tree.clone_from_tree(&tree);
                        }
                        _ => {
                            config.trees.insert(tree.get_name().to_string(), tree);
                        }
                    }
                }
            }
            true
        }
        _ => false,
    }
}

/// Return a tree from a simple "tree: <url>" entry
fn get_tree_from_url(name: &Yaml, url: &str) -> model::Tree {
    let mut tree = model::Tree::default();

    // Tree name
    get_str(name, tree.get_name_mut());

    // Default to the name when "path" is unspecified.
    let tree_name = tree.get_name().to_string();
    tree.get_path_mut().set_expr(tree_name.to_string());
    tree.get_path().set_value(tree_name);

    if syntax::is_git_dir(tree.get_path().get_expr()) {
        tree.is_bare_repository = true;
    }

    // Register the ${TREE_NAME} variable.
    tree.variables.insert(
        string!("TREE_NAME"),
        model::Variable::new(tree.get_name().clone(), None),
    );

    // Register the ${TREE_PATH} variable.
    tree.variables.insert(
        string!("TREE_PATH"),
        model::Variable::new(tree.get_path().get_expr().clone(), None),
    );

    tree.remotes.insert(
        string!("origin"),
        model::Variable::new(url.to_string(), None),
    );

    tree
}

/// Read fields common to trees and templates.
#[inline]
fn get_tree_fields(value: &Yaml, tree: &mut model::Tree) {
    get_variables_hashmap(&value["variables"], &mut tree.variables);
    get_multivariables_hashmap(&value["gitconfig"], &mut tree.gitconfig);
    get_str_variables_hashmap(&value["remotes"], &mut tree.remotes);

    get_multivariables(&value["environment"], &mut tree.environment);
    get_multivariables_hashmap(&value["commands"], &mut tree.commands);

    get_variable(&value["branch"], &mut tree.branch);
    get_variables_hashmap(&value["branches"], &mut tree.branches);
    get_variable(&value["symlink"], &mut tree.symlink);
    get_variable(&value["worktree"], &mut tree.worktree);

    get_i64(&value["depth"], &mut tree.clone_depth);
    get_bool(&value["bare"], &mut tree.is_bare_repository);
    get_bool(&value["single-branch"], &mut tree.is_single_branch);

    tree.update_flags();
}

/// Read a single tree definition
fn get_tree(
    app_context: &model::ApplicationContext,
    config: &mut model::Configuration,
    name: &Yaml,
    value: &Yaml,
    trees: &YamlHash,
    variables: bool,
) -> model::Tree {
    // The tree that will be built and returned.
    let mut tree = model::Tree::default();

    // Allow extending an existing tree by specifying "extend".
    let mut extend = String::new();
    if get_str(&value["extend"], &mut extend) {
        // Holds a base tree specified using "extend: <tree>".
        let tree_name = Yaml::String(extend.clone());
        if let Some(tree_values) = trees.get(&tree_name) {
            let base_tree = get_tree(app_context, config, &tree_name, tree_values, trees, false);
            tree.clone_from_tree(&base_tree);
        } else {
            // Allow the referenced tree to be found from an earlier include.
            if let Some(base) = config.get_tree(&extend) {
                tree.clone_from_tree(base);
            }
        }
        tree.templates.truncate(0); // Base templates were already processed.
    }

    // Load values from the parent tree when using "worktree: <parent>".
    let mut parent_expr = String::new();
    if get_str(&value["worktree"], &mut parent_expr) {
        let parent_name = eval::value(app_context, config, &parent_expr);
        if !parent_expr.is_empty() {
            let tree_name = Yaml::String(parent_name);
            if let Some(tree_values) = trees.get(&tree_name) {
                let base = get_tree(app_context, config, &tree_name, tree_values, trees, true);
                tree.clone_from_tree(&base);
            }
        }
        tree.templates.truncate(0); // Base templates were already processed.
    }

    // Templates
    // Process the base templates in the specified order before processing
    // the template itself.
    get_indexset_str(&value["templates"], &mut tree.templates);
    for template_name in &tree.templates.clone() {
        // Do we have a template by this name? If so, apply the template.
        if let Some(template) = config.templates.get(template_name) {
            template.apply(&mut tree);
        }
    }

    // Tree name
    get_str(name, tree.get_name_mut());

    // Tree path
    if !get_str(&value["path"], tree.get_path_mut().get_expr_mut()) {
        // Default to the name when "path" is unspecified.
        let tree_name = tree.get_name().to_string();
        tree.get_path_mut().set_expr(tree_name.to_string());
        tree.get_path().set_value(tree_name);
    }

    // Detect bare repositories.
    if syntax::is_git_dir(tree.get_path().get_expr()) {
        tree.is_bare_repository = true;
    }

    // Add the TREE_NAME and TREE_PATH variables
    if variables {
        // Register the ${TREE_NAME} variable.
        tree.variables.insert(
            string!("TREE_NAME"),
            model::Variable::new(tree.get_name().clone(), None),
        );
        // Register the ${TREE_PATH} variable.
        tree.variables.insert(
            string!("TREE_PATH"),
            model::Variable::new(tree.get_path().get_expr().clone(), None),
        );
    }

    // Load the URL and store it in the "origin" remote.
    {
        let mut url = String::new();
        if get_str(&value["url"], &mut url) {
            tree.remotes
                .insert(string!("origin"), model::Variable::new(url, None));
        }
    }

    get_tree_fields(value, &mut tree);

    tree
}

/// Read simple string values into a garden::model::VariableHashMap.
fn get_str_variables_hashmap(yaml: &Yaml, remotes: &mut model::VariableHashMap) {
    if let Yaml::Hash(hash) = yaml {
        for (name, value) in hash {
            if let (Some(name_str), Some(value_str)) = (name.as_str(), value.as_str()) {
                remotes.insert(
                    name_str.to_string(),
                    model::Variable::new(value_str.to_string(), None),
                );
            }
        }
    }
}

/// Read group definitions
fn get_groups(yaml: &Yaml, groups: &mut IndexMap<model::GroupName, model::Group>) -> bool {
    match yaml {
        Yaml::Hash(hash) => {
            for (name, value) in hash {
                let mut group = model::Group::default();
                get_str(name, group.get_name_mut());
                get_indexset_str(value, &mut group.members);
                groups.insert(group.get_name_owned(), group);
            }
            true
        }
        _ => false,
    }
}

/// Read garden definitions
fn get_gardens(yaml: &Yaml, gardens: &mut IndexMap<String, model::Garden>) -> bool {
    match yaml {
        Yaml::Hash(hash) => {
            for (name, value) in hash {
                let mut garden = model::Garden::default();
                get_str(name, garden.get_name_mut());
                get_indexset_str(&value["groups"], &mut garden.groups);
                get_indexset_str(&value["trees"], &mut garden.trees);
                get_multivariables_hashmap(&value["gitconfig"], &mut garden.gitconfig);
                get_variables_hashmap(&value["variables"], &mut garden.variables);
                get_multivariables(&value["environment"], &mut garden.environment);
                get_multivariables_hashmap(&value["commands"], &mut garden.commands);
                gardens.insert(garden.get_name().to_string(), garden);
            }
            true
        }
        _ => false,
    }
}

/// Read a grafts: block into a Vec<Graft>.
fn get_grafts(yaml: &Yaml, grafts: &mut IndexMap<model::GardenName, model::Graft>) -> bool {
    match yaml {
        Yaml::Hash(yaml_hash) => {
            for (name, value) in yaml_hash {
                let graft = get_graft(name, value);
                grafts.insert(graft.get_name().to_string(), graft);
            }
            true
        }
        _ => false,
    }
}

fn get_graft(name: &Yaml, graft: &Yaml) -> model::Graft {
    let mut graft_name = string!("");
    let mut config = string!("");
    let mut root = string!("");

    get_str(name, &mut graft_name);

    if !get_str(graft, &mut config) {
        // The root was not specified.
        if let Yaml::Hash(_hash) = graft {
            // A config expression and root might be specified.
            get_str(&graft["config"], &mut config);
            get_str(&graft["root"], &mut root);
        }
    }

    model::Graft::new(graft_name, root, config)
}

/// Read and parse YAML from a file path.
pub fn read_yaml<P>(path: P) -> Result<Yaml, errors::GardenError>
where
    P: std::convert::AsRef<std::path::Path> + std::fmt::Debug,
{
    let string =
        std::fs::read_to_string(&path).map_err(|io_err| errors::GardenError::ReadFile {
            path: path.as_ref().into(),
            err: io_err,
        })?;

    let mut docs =
        YamlLoader::load_from_str(&string).map_err(|err| errors::GardenError::ReadConfig {
            err,
            path: path.as_ref().display().to_string(),
        })?;

    if docs.is_empty() {
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
        if let Yaml::Hash(doc_hash) = doc {
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
        if let Yaml::Hash(doc_hash) = doc {
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
        if let Yaml::Hash(doc_hash) = doc {
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
        if let Yaml::Hash(doc_hash) = doc {
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
    add_missing_sections(&mut doc).unwrap_or(());

    doc
}
