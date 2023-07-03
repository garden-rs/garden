use super::cli;
use super::collections::{append_hashmap, append_indexset};
use super::config;
use super::errors;
use super::eval;
use super::path;
use super::syntax;

use clap::ValueEnum;
use indexmap::{IndexMap, IndexSet};
use indextree::{Arena, NodeId};
use std::cell::RefCell;
use std::collections::HashMap;
use which::which;

/// TreeName keys into config.trees
pub type TreeName = String;

/// GroupName keys into config.groups
pub type GroupName = String;

/// GardenName keys into config.gardens
pub type GardenName = String;

/// GraftName keys into config.grafts
pub type GraftName = String;

/// Configuration Node IDs
pub type ConfigId = NodeId;

/// Config files can define a sequence of variables that are
/// iteratively calculated.  Variables can reference other
/// variables in their Tree, Garden, and Configuration scopes.
///
/// The config values can contain either plain values,
/// string ${expressions} that resolve against other Variables,
/// or exec expressions that evaluate to a command whose stdout is
/// captured and placed into the value of the variable.
///
/// An exec expression can use shell-like ${variable} references as which
/// are substituted when evaluating the command, just like a regular
/// string expression.  An exec expression is denoted by using a "$ "
/// (dollar-sign followed by space) before the value.  For example,
/// using "$ echo foo" will place the value "foo" in the variable.
#[derive(Clone, Debug, Default)]
pub struct Variable {
    expr: String,
    value: RefCell<Option<String>>,
}

impl_display_brief!(Variable);

impl Variable {
    pub fn new(expr: String, value: Option<String>) -> Self {
        Variable {
            expr,
            value: RefCell::new(value),
        }
    }

    /// Does this variable have a value?
    pub fn is_empty(&self) -> bool {
        self.expr.is_empty()
    }

    pub fn get_expr(&self) -> &String {
        &self.expr
    }

    pub fn get_expr_mut(&mut self) -> &mut String {
        &mut self.expr
    }

    pub fn set_expr(&mut self, expr: String) {
        self.expr = expr;
    }

    pub fn set_value(&self, value: String) {
        *self.value.borrow_mut() = Some(value);
    }

    /// Transform the `RefCell<Option<String>>` value into `Option<&String>`.
    pub fn get_value(&self) -> Option<&String> {
        let ptr = self.value.as_ptr();
        unsafe { (*ptr).as_ref() }
    }

    pub fn reset(&self) {
        *self.value.borrow_mut() = None;
    }
}

/// An unordered mapping of names to a vector of Variables.
pub type MultiVariableHashMap = HashMap<String, Vec<Variable>>;

/// Reset the variables held inside a MultiVariableHashMap.
fn reset_hashmap_variables(vec_variables: &MultiVariableHashMap) {
    for variables in vec_variables.values() {
        for variable in variables {
            variable.reset();
        }
    }
}

/// An unordered mapping of name to Variable.
pub type VariableHashMap = HashMap<String, Variable>;

// Named variables with a single value
#[derive(Clone, Debug)]
pub struct NamedVariable {
    name: String,
    variable: Variable,
}

impl_display_brief!(NamedVariable);

impl NamedVariable {
    pub fn new(name: String, expr: String, value: Option<String>) -> Self {
        NamedVariable {
            name,
            variable: Variable::new(expr, value),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_expr(&self) -> &String {
        self.variable.get_expr()
    }

    pub fn set_expr(&mut self, expr: String) {
        self.variable.set_expr(expr);
    }

    pub fn set_value(&self, value: String) {
        self.variable.set_value(value);
    }

    pub fn get_value(&self) -> Option<&String> {
        self.variable.get_value()
    }

    pub fn reset(&self) {
        self.variable.reset();
    }
}

// Named variables with multiple values
#[derive(Clone, Debug)]
pub struct MultiVariable {
    name: String,
    variables: Vec<Variable>,
}

impl_display!(MultiVariable);

impl MultiVariable {
    pub fn new(name: String, variables: Vec<Variable>) -> Self {
        MultiVariable { name, variables }
    }

    pub fn get(&self, idx: usize) -> &Variable {
        &self.variables[idx]
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn len(&self) -> usize {
        self.variables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    pub fn reset(&self) {
        for var in &self.variables {
            var.reset();
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Variable> {
        self.variables.iter()
    }
}

/// Trees represent a single worktree
#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub commands: MultiVariableHashMap,
    pub environment: Vec<MultiVariable>,
    pub gitconfig: VariableHashMap,
    pub remotes: VariableHashMap,
    pub symlink: Variable,
    pub templates: IndexSet<String>,
    pub variables: VariableHashMap,
    pub branch: Variable,
    pub branches: VariableHashMap,
    pub worktree: Variable,
    pub clone_depth: i64,
    pub is_single_branch: bool,
    pub is_symlink: bool,
    pub is_bare_repository: bool,
    pub is_worktree: bool,

    name: String,
    path: Variable,
}

impl_display!(Tree);

impl Tree {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn get_path(&self) -> &Variable {
        &self.path
    }

    pub fn get_path_mut(&mut self) -> &mut Variable {
        &mut self.path
    }

    pub fn path_is_valid(&self) -> bool {
        self.path.get_value().is_some()
    }

    /// Build a canonicalized pathbuf for the current tree.
    pub fn canonical_pathbuf(&self) -> Option<std::path::PathBuf> {
        if let Some(pathbuf) = self.pathbuf() {
            if let Ok(canon_path) = pathbuf.canonicalize() {
                return Some(canon_path);
            }
        }

        None
    }

    /// Build a pathbuf for the current tree.
    pub fn pathbuf(&self) -> Option<std::path::PathBuf> {
        if !self.path_is_valid() {
            return None;
        }
        self.path.get_value().map(std::path::PathBuf::from)
    }

    pub fn path_as_ref(&self) -> Result<&String, errors::GardenError> {
        match self.path.get_value() {
            Some(value) => Ok(value),
            None => Err(errors::GardenError::ConfigurationError(format!(
                "unset tree path for {}",
                self.name
            ))),
        }
    }

    pub fn symlink_as_ref(&self) -> Result<&String, errors::GardenError> {
        match self.symlink.get_value() {
            Some(value) => Ok(value),
            None => Err(errors::GardenError::ConfigurationError(format!(
                "unset tree path for {}",
                self.name
            ))),
        }
    }

    pub fn reset_variables(&self) {
        // self.path is a variable but it is not reset because
        // the tree path is evaluated once when the configuration
        // is first read, and never again.
        for var in self.variables.values() {
            var.reset();
        }

        for cfg in self.gitconfig.values() {
            cfg.reset();
        }

        for env in &self.environment {
            env.reset();
        }

        reset_hashmap_variables(&self.commands);
    }

    /// Copy the guts of another tree into the current tree.
    pub fn clone_from_tree(&mut self, tree: &Tree) {
        append_hashmap(&mut self.commands, &tree.commands);
        append_hashmap(&mut self.gitconfig, &tree.gitconfig);
        append_hashmap(&mut self.variables, &tree.variables);
        append_hashmap(&mut self.remotes, &tree.remotes);
        append_indexset(&mut self.templates, &tree.templates);

        // "environment" follow last-set-wins semantics.
        self.environment.append(&mut tree.environment.clone());

        // The last value set is the one that wins.
        if tree.clone_depth > 0 {
            self.clone_depth = tree.clone_depth;
        }

        if tree.is_bare_repository {
            self.is_bare_repository = tree.is_bare_repository;
        }

        if tree.is_single_branch {
            self.is_single_branch = tree.is_single_branch;
        }

        if tree.is_worktree {
            self.is_worktree = tree.is_worktree;
        }
        if tree.is_symlink {
            self.is_symlink = tree.is_symlink;
        }

        if !tree.branch.is_empty() {
            self.branch = tree.branch.clone();
        }

        if !tree.symlink.is_empty() {
            self.symlink = tree.symlink.clone();
        }

        if !tree.worktree.is_empty() {
            self.worktree = tree.worktree.clone();
        }

        self.update_flags();
    }

    /// Update internal flags in response to newly read data.
    pub fn update_flags(&mut self) {
        if !self.symlink.is_empty() {
            self.is_symlink = true;
        }
        if !self.worktree.is_empty() {
            self.is_worktree = true;
        }
        if self.is_worktree {
            self.is_bare_repository = false;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Group {
    name: String,
    pub members: IndexSet<String>,
}

impl_display!(Group);

impl Group {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Return an owned copy of the name field.
    pub fn get_name_owned(&self) -> String {
        self.get_name().to_owned()
    }

    pub fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }
}

/// Templates can be used to create trees.
/// They contain a (path-less) tree object which can be used for creating
/// materialized trees.
#[derive(Clone, Debug, Default)]
pub struct Template {
    pub tree: Tree,
    pub extend: IndexSet<String>,
    name: String,
}

impl_display!(Template);

impl Template {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    /// Apply this template onto the specified tree.
    pub fn apply(&self, tree: &mut Tree) {
        tree.clone_from_tree(&self.tree);
    }
}

// Gardens aggregate trees
#[derive(Clone, Debug, Default)]
pub struct Garden {
    pub commands: MultiVariableHashMap,
    pub environment: Vec<MultiVariable>,
    pub gitconfig: VariableHashMap,
    pub groups: IndexSet<String>,
    pub trees: IndexSet<String>,
    pub variables: VariableHashMap,
    name: GardenName,
}

impl_display!(Garden);

impl Garden {
    pub fn get_name(&self) -> &GardenName {
        &self.name
    }

    pub fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }
}

/// Return the default shell to use for custom commands and "garden shell".
fn get_default_shell() -> String {
    if which("zsh").is_ok() {
        "zsh"
    } else if which("bash").is_ok() {
        "bash"
    } else {
        "sh"
    }
    .to_string()
}

// Configuration represents an instantiated garden configuration
#[derive(Clone, Debug, Default)]
pub struct Configuration {
    pub commands: MultiVariableHashMap,
    pub debug: HashMap<String, u8>,
    pub environment: Vec<MultiVariable>,
    pub gardens: IndexMap<GardenName, Garden>,
    pub grafts: IndexMap<GraftName, Graft>,
    pub groups: IndexMap<GroupName, Group>,
    pub path: Option<std::path::PathBuf>,
    pub dirname: Option<std::path::PathBuf>,
    pub root: Variable,
    pub root_path: std::path::PathBuf,
    pub shell: String,
    pub templates: HashMap<String, Template>,
    pub tree_search_path: Vec<std::path::PathBuf>,
    pub trees: IndexMap<TreeName, Tree>,
    pub variables: VariableHashMap,
    pub verbose: u8,
    pub parent_id: Option<ConfigId>,
    id: Option<ConfigId>,
}

impl_display!(Configuration);

impl Configuration {
    /// Create a default Configuration
    pub fn new() -> Self {
        Configuration {
            id: None,
            parent_id: None,
            shell: get_default_shell(),
            ..std::default::Default::default()
        }
    }

    pub fn initialize(&mut self, app_context: &ApplicationContext) {
        // Evaluate garden.root
        let expr = String::from(self.root.get_expr());
        let value = eval::value(app_context, self, &expr);
        // Store the resolved, canonicalized garden.root
        self.root_path = std::path::PathBuf::from(&value);
        if let Ok(root_path_canon) = self.root_path.canonicalize() {
            self.root_path = root_path_canon;
        }
        self.root.set_value(value);

        // Resolve tree paths
        self.update_tree_paths(app_context);

        // Reset variables
        self.reset();
    }

    pub fn update(
        &mut self,
        app_context: &ApplicationContext,
        config: Option<&std::path::PathBuf>,
        root: Option<&std::path::PathBuf>,
        config_verbose: u8,
        parent: Option<ConfigId>,
    ) -> Result<(), errors::GardenError> {
        if let Some(parent_id) = parent {
            self.set_parent(parent_id);
        }
        self.verbose = config_verbose;

        // Override the configured garden root
        if let Some(root_path) = root {
            self.root.set_expr(root_path.to_string_lossy().to_string());
        }

        let mut basename: String = "garden.yaml".into();

        // Find garden.yaml in the search path
        let mut found = false;
        if let Some(config_path) = config {
            if config_path.is_file() || config_path.is_absolute() {
                // If an absolute path was specified, or if the file exists,
                // short-circuit the search; the config file might be missing but
                // we shouldn't silently use a different config file.
                self.set_path(config_path.to_path_buf());
                found = true;
            } else {
                // The specified path is a basename or relative path to be found
                // in the config search path.
                basename = config_path.to_string_lossy().into();
            }
        }

        if !found {
            for entry in config::search_path() {
                let mut candidate = entry.to_path_buf();
                candidate.push(basename.clone());
                if candidate.exists() {
                    self.set_path(candidate);
                    found = true;
                    break;
                }
            }
        }
        if config_verbose > 0 {
            debug!(
                "config: path: {:?}, root: {:?}, found: {}",
                self.path, self.root, found
            );
        }

        if found {
            // Read file contents.
            let config_path = self.get_path()?;
            if let Ok(config_string) = std::fs::read_to_string(config_path) {
                config::parse(app_context, &config_string, config_verbose, self)?;
            } else {
                // Return a default Configuration If we are unable to read the file.
                return Ok(());
            }
        }

        // Default to the current directory when garden.root is unspecified
        if self.root.get_expr().is_empty() {
            self.root.set_expr(path::current_dir_string());
        }

        Ok(())
    }

    /// Apply MainOptions to a Configuration.
    pub fn update_options(
        &mut self,
        options: &cli::MainOptions,
    ) -> Result<(), errors::GardenError> {
        let config_verbose = options.debug_level("config");
        if self.path.is_none() {
            error!("unable to find a configuration file -- use --config <path>");
        }
        if config_verbose > 1 {
            eprintln!("config: {:?}", self.get_path()?);
        }
        if config_verbose > 2 {
            debug!("{}", self);
        }

        for key in &options.debug {
            let current = *self.debug.get(key).unwrap_or(&0);
            self.debug.insert(key.into(), current + 1);
        }

        for k_eq_v in &options.define {
            let name: String;
            let expr: String;
            let values: Vec<&str> = k_eq_v.splitn(2, '=').collect();
            if values.len() == 1 {
                name = values[0].to_string();
                expr = string!("");
            } else if values.len() == 2 {
                name = values[0].to_string();
                expr = values[1].to_string();
            } else {
                error!("unable to split '{}'", k_eq_v);
            }
            self.variables.insert(name, Variable::new(expr, None));
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        // Reset variables to allow for tree-scope evaluation
        self.reset_variables();

        // Add custom variables
        self.reset_builtin_variables()
    }

    fn reset_builtin_variables(&mut self) {
        // Update GARDEN_ROOT.
        if let Some(var) = self.variables.get_mut("GARDEN_ROOT") {
            if let Some(value) = self.root.get_value() {
                var.set_expr(value.into());
                var.set_value(value.into());
            }
        }

        for tree in self.trees.values_mut() {
            // Update TREE_NAME.
            let tree_name = String::from(tree.get_name());
            if let Some(var) = tree.variables.get_mut("TREE_NAME") {
                var.set_expr(tree_name.to_string());
                var.set_value(tree_name);
            }
            // Extract the tree's path.  Skip invalid/unset entries.
            let tree_path = match tree.path_as_ref() {
                Ok(path) => String::from(path),
                Err(_) => continue,
            };
            // Update TREE_PATH.
            if let Some(var) = tree.variables.get_mut("TREE_PATH") {
                var.set_expr(tree_path.to_string());
                var.set_value(tree_path);
            }
        }
    }

    // Calculate the "path" field for each tree.
    // If specified as a relative path, it will be relative to garden.root.
    // If specified as an asbolute path, it will be left as-is.
    fn update_tree_paths(&mut self, app_context: &ApplicationContext) {
        // Gather path and symlink expressions.
        let mut path_values = Vec::new();
        let mut symlink_values = Vec::new();

        for (name, tree) in &self.trees {
            path_values.push((name.clone(), tree.path.get_expr().clone()));
            if tree.is_symlink {
                symlink_values.push((name.clone(), tree.symlink.get_expr().clone()));
            }
        }

        // Evaluate the "path" expression.
        for (name, value) in &path_values {
            let result = self.eval_tree_path(app_context, value);
            if let Some(tree) = self.trees.get_mut(name) {
                tree.path.set_value(result);
            }
        }

        // Evaluate the "symlink" expression.
        for (name, value) in &symlink_values {
            let result = self.eval_tree_path(app_context, value);
            if let Some(tree) = self.trees.get_mut(name) {
                tree.symlink.set_value(result);
            }
        }
    }

    /// Return a path string relative to the garden root
    pub fn tree_path(&self, path: &str) -> String {
        if std::path::PathBuf::from(path).is_absolute() {
            // Absolute path, nothing to do
            path.into()
        } else {
            // Make path relative to root_path
            let mut path_buf = self.root_path.to_path_buf();
            path_buf.push(path);

            path_buf.to_string_lossy().into()
        }
    }

    /// Return a pathbuf relative to the garden root.
    pub fn relative_pathbuf(&self, path: &str) -> std::path::PathBuf {
        let pathbuf = std::path::PathBuf::from(path);
        if pathbuf.is_absolute() {
            // Absolute path, nothing to do
            if let Ok(pathbuf_canon) = pathbuf.canonicalize() {
                pathbuf_canon
            } else {
                pathbuf
            }
        } else {
            // Make path relative to root_path
            let mut path_buf = self.root_path.to_path_buf();
            path_buf.push(path);

            path_buf
        }
    }

    /// Evaluate and return a path string relative to the garden root.
    pub fn eval_tree_path(&mut self, app_context: &ApplicationContext, path: &str) -> String {
        let value = eval::value(app_context, self, path);
        self.tree_path(&value)
    }

    /// Resolve a pathbuf relative to the config directory.
    pub fn config_pathbuf(&self, path: &str) -> Option<std::path::PathBuf> {
        let path_buf = std::path::PathBuf::from(path);
        if path_buf.is_absolute() {
            // Absolute path, nothing to do
            Some(path_buf)
        } else if let Some(dirname) = self.dirname.as_ref() {
            // Anchor relative paths to the configuration's dirname.
            let mut abs_path_buf = dirname.to_path_buf();
            abs_path_buf.push(path_buf);

            Some(abs_path_buf)
        } else {
            None
        }
    }

    /// Resolve a pathbuf relative to specified include file or the config directory.
    /// Returns the first file found. The include file's directory is checked first.
    pub fn config_pathbuf_from_include(
        &self,
        include_path: &std::path::Path,
        path: &str,
    ) -> Option<std::path::PathBuf> {
        let mut path_buf = std::path::PathBuf::from(path);
        if path_buf.is_absolute() {
            // Absolute path, nothing to do
            return Some(path_buf);
        }

        // First check if the path exists relative to the specified parent directory.
        if let Some(dirname) = include_path.parent() {
            // Make path relative to the include file.
            path_buf = dirname.to_path_buf();
            path_buf.push(path);

            if path_buf.exists() {
                return Some(path_buf);
            }
        }

        // Make path relative to the configuration's dirname
        self.config_pathbuf(path)
    }

    /// Resolve a path string relative to the config directory.
    pub fn config_path(&self, path: &str) -> String {
        if let Some(path_buf) = self.config_pathbuf(path) {
            path_buf.to_string_lossy().to_string()
        } else {
            self.tree_path(path)
        }
    }

    /// Evaluate and resolve a path string and relative to the config directory.
    pub fn eval_config_path(&self, app_context: &ApplicationContext, path: &str) -> String {
        let value = eval::value(app_context, self, path);
        self.config_path(&value)
    }

    /// Evaluate and resolve a pathbuf relative to the config directory.
    pub fn eval_config_pathbuf(
        &self,
        app_context: &ApplicationContext,
        path: &str,
    ) -> Option<std::path::PathBuf> {
        let value = eval::value(app_context, self, path);
        self.config_pathbuf(&value)
    }

    /// Evaluate and resolve a pathbuf relative to the config directory for "includes".
    pub fn eval_config_pathbuf_from_include(
        &self,
        app_context: &ApplicationContext,
        include_path: Option<&std::path::Path>,
        path: &str,
    ) -> Option<std::path::PathBuf> {
        let value = eval::value(app_context, self, path);

        if let Some(include_path) = include_path {
            self.config_pathbuf_from_include(include_path, &value)
        } else {
            self.config_pathbuf(&value)
        }
        .or_else(|| Some(std::path::PathBuf::from(&value)))
    }

    /// Reset resolved variables
    pub fn reset_variables(&mut self) {
        for var in self.variables.values() {
            var.reset();
        }
        for env in &self.environment {
            env.reset();
        }

        reset_hashmap_variables(&self.commands);

        for tree in self.trees.values() {
            tree.reset_variables();
        }
    }

    /// Set the ConfigId from the Arena for this configuration.
    pub fn set_id(&mut self, id: ConfigId) {
        self.id = Some(id);
    }

    pub fn get_id(&self) -> Option<ConfigId> {
        self.id
    }

    /// Set the parent ConfigId from the Arena for this configuration.
    pub fn set_parent(&mut self, id: ConfigId) {
        self.parent_id = Some(id);
    }

    /// Set the config path and the dirname fields
    pub fn set_path(&mut self, path: std::path::PathBuf) {
        let mut dirname = path.clone();
        dirname.pop();

        self.dirname = Some(dirname);
        self.path = Some(path);
    }

    /// Get the config path if it is defined.
    pub fn get_path(&self) -> Result<&std::path::PathBuf, errors::GardenError> {
        self.path
            .as_ref()
            .ok_or_else(|| errors::GardenError::AssertionError("cfg.path is unset".into()))
    }

    /// Get a path string for this configuration.
    /// Returns the current directory when the configuration does not have a valid path.
    pub fn get_path_for_display(&self) -> String {
        let default_pathbuf = std::path::PathBuf::from(".");
        self.path
            .as_ref()
            .unwrap_or(&default_pathbuf)
            .display()
            .to_string()
    }

    /// Return true if the configuration contains the named graft.
    pub fn contains_graft(&self, name: &str) -> bool {
        let graft_name = syntax::trim(name);
        self.grafts.contains_key(graft_name)
    }

    /// Return a graft by name.
    pub fn get_graft(&self, name: &str) -> Result<&Graft, errors::GardenError> {
        let graft_name = syntax::trim(name);
        self.grafts.get(graft_name).ok_or_else(|| {
            errors::GardenError::ConfigurationError(format!("{name}: no such graft"))
        })
    }

    /// Parse a "graft::value" string and return the ConfigId for the graft and the
    /// remaining unparsed "value".
    pub fn get_graft_id<'a>(
        &self,
        value: &'a str,
    ) -> Result<(ConfigId, &'a str), errors::GardenError> {
        let (ok, graft_name, remainder) = syntax::split_graft(value);
        if !ok {
            return Err(errors::GardenError::ConfigurationError(format!(
                "{value}: invalid graft expression"
            )));
        }
        let graft = self.get_graft(graft_name)?;
        let graft_id = graft
            .get_id()
            .ok_or(errors::GardenError::ConfigurationError(format!(
                "{graft_name}: no such graft"
            )))?;

        Ok((graft_id, remainder))
    }

    /// Find a tree by name and return a reference if it exists.
    pub fn get_tree(&self, name: &str) -> Option<&Tree> {
        self.trees.get(name)
    }

    /// Return a pathbuf for the specified Tree index
    pub fn get_tree_pathbuf(&self, tree_name: &str) -> Option<std::path::PathBuf> {
        self.get_tree(tree_name)
            .map(|tree| tree.canonical_pathbuf())
            .unwrap_or(None)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Graft {
    id: Option<ConfigId>,
    name: String,
    pub root: String,
    pub config: String,
}

impl_display!(Graft);

impl Graft {
    pub fn new(name: String, root: String, config: String) -> Self {
        Graft {
            id: None,
            name,
            root,
            config,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_id(&self) -> Option<ConfigId> {
        self.id
    }

    pub fn set_id(&mut self, id: ConfigId) {
        self.id = Some(id);
    }
}

// TODO EvalContext
#[derive(Clone, Debug)]
pub struct EvalContext {
    pub config: ConfigId,
    pub tree: Option<TreeName>,
    pub garden: Option<GardenName>,
    pub group: Option<GroupName>,
}

impl_display_brief!(EvalContext);

impl EvalContext {
    /// Construct a new EvalContext.
    pub fn new(
        config: ConfigId,
        tree: Option<TreeName>,
        garden: Option<GardenName>,
        group: Option<GroupName>,
    ) -> Self {
        EvalContext {
            config,
            tree,
            garden,
            group,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeContext {
    pub tree: TreeName,
    pub config: Option<ConfigId>,
    pub garden: Option<GardenName>,
    pub group: Option<String>,
}

impl_display_brief!(TreeContext);

impl TreeContext {
    /// Construct a new TreeContext.
    pub fn new(
        tree: &str,
        config: Option<ConfigId>,
        garden: Option<GardenName>,
        group: Option<String>,
    ) -> Self {
        TreeContext {
            tree: TreeName::from(tree),
            config,
            garden,
            group,
        }
    }
}

#[derive(Debug, Default)]
pub struct TreeQuery {
    pub query: String,
    pub pattern: glob::Pattern,
    pub is_default: bool,
    pub is_garden: bool,
    pub is_group: bool,
    pub is_tree: bool,
    pub include_gardens: bool,
    pub include_groups: bool,
    pub include_trees: bool,
}

impl_display_brief!(TreeQuery);

impl TreeQuery {
    pub fn new(query: &str) -> Self {
        let mut is_default = false;
        let mut is_tree = false;
        let mut is_garden = false;
        let mut is_group = false;
        let mut include_gardens = true;
        let mut include_groups = true;
        let mut include_trees = true;

        if syntax::is_garden(query) {
            is_garden = true;
            include_groups = false;
            include_trees = false;
        } else if syntax::is_group(query) {
            is_group = true;
            include_gardens = false;
            include_trees = false;
        } else if syntax::is_tree(query) {
            is_tree = true;
            include_gardens = false;
            include_groups = false;
        } else {
            is_default = true;
        }
        let glob_pattern = syntax::trim(query);
        let pattern = glob::Pattern::new(glob_pattern).unwrap_or_default();

        TreeQuery {
            query: query.into(),
            is_default,
            is_garden,
            is_group,
            is_tree,
            include_gardens,
            include_groups,
            include_trees,
            pattern,
        }
    }
}

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Enable color when a tty is detected
    #[default]
    Auto,
    /// Disable color
    Off,
    /// Enable color
    On,
}

impl ColorMode {
    pub fn is_enabled(&self) -> bool {
        match self {
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
            ColorMode::Off => false,
            ColorMode::On => true,
        }
    }

    pub fn names() -> &'static str {
        "auto, true, false, 1, 0, [y]es, [n]o, on, off, always, never"
    }

    pub fn update(&mut self) {
        if *self == ColorMode::Auto {
            // Speedup future calls to is_enabled() by performing the "auto"
            // atty check once and caching the result.
            if self.is_enabled() {
                *self = ColorMode::On;
            } else {
                *self = ColorMode::Off;
            }
        }

        if *self == ColorMode::Off {
            yansi::Paint::disable();
        }
    }
}

impl std::str::FromStr for ColorMode {
    type Err = (); // For the FromStr trait

    fn from_str(src: &str) -> Result<ColorMode, ()> {
        match src.to_lowercase().as_ref() {
            "auto" => Ok(ColorMode::Auto),
            "-1" => Ok(ColorMode::Auto),
            "0" => Ok(ColorMode::Off),
            "1" => Ok(ColorMode::On),
            "false" => Ok(ColorMode::Off),
            "true" => Ok(ColorMode::On),
            "never" => Ok(ColorMode::Off),
            "always" => Ok(ColorMode::Off),
            "off" => Ok(ColorMode::Off),
            "on" => Ok(ColorMode::On),
            "n" => Ok(ColorMode::Off),
            "y" => Ok(ColorMode::On),
            "no" => Ok(ColorMode::Off),
            "yes" => Ok(ColorMode::On),
            _ => Err(()),
        }
    }
}

// Color is an alias for yansi::Paint.
pub type Color<T> = yansi::Paint<T>;

pub fn display_missing_tree(tree: &Tree, path: &str, verbose: u8) -> String {
    if verbose > 0 {
        format!(
            "{} {}  {} {}",
            Color::black("#").bold(),
            Color::black(&tree.name).bold(),
            Color::black(&path).bold(),
            Color::black("(skipped)").bold()
        )
    } else {
        format!(
            "{} {} {}",
            Color::black("#").bold(),
            Color::black(&tree.name).bold(),
            Color::black("(skipped)").bold()
        )
    }
}

pub fn display_tree(tree: &Tree, path: &str, verbose: u8) -> String {
    if verbose > 0 {
        format!(
            "{} {}  {}",
            Color::cyan("#"),
            Color::blue(&tree.name).bold(),
            Color::blue(&path)
        )
    } else {
        format!("{} {}", Color::cyan("#"), Color::blue(&tree.name).bold())
    }
}

/// Print a tree if it exists, otherwise print a missing tree
pub fn print_tree(tree: &Tree, verbose: u8, quiet: bool) -> bool {
    if let Ok(path) = tree.path_as_ref() {
        // Sparse gardens/missing trees are ok -> skip these entries.
        if !std::path::PathBuf::from(&path).exists() {
            if !quiet {
                eprintln!("{}", display_missing_tree(tree, path, verbose));
            }
            return false;
        }

        print_tree_details(tree, verbose, quiet);
        return true;
    } else if !quiet {
        eprintln!("{}", display_missing_tree(tree, "[invalid-path]", verbose));
    }

    false
}

/// Print a tree
pub fn print_tree_details(tree: &Tree, verbose: u8, quiet: bool) {
    if !quiet {
        if let Ok(path) = tree.path_as_ref() {
            eprintln!("{}", display_tree(tree, path, verbose));
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApplicationContext {
    pub options: cli::MainOptions,
    arena: RefCell<Arena<Configuration>>,
    root_id: ConfigId,
}

impl_display!(ApplicationContext);

impl ApplicationContext {
    /// Construct an empty ApplicationContext. Initialization must be performed post-construction.
    pub fn new(options: cli::MainOptions) -> Self {
        let mut arena = Arena::new();
        let config = Configuration::new();
        let root_id = arena.new_node(config);

        let app_context = ApplicationContext {
            arena: RefCell::new(arena),
            root_id,
            options,
        };
        // Record the ID in the configuration.
        let config = app_context.get_root_config_mut();
        config.set_id(root_id);

        app_context
    }

    /// Initialize an ApplicationContext and Configuration from cli::MainOptions.
    pub fn from_options(options: &cli::MainOptions) -> Result<Self, errors::GardenError> {
        let app_context = Self::new(options.clone());
        let config_verbose = options.debug_level("config");
        app_context.get_root_config_mut().update(
            &app_context,
            options.config.as_ref(),
            options.root.as_ref(),
            config_verbose,
            None,
        )?;

        app_context.get_root_config_mut().update_options(options)?;

        config::read_grafts(&app_context)?;

        Ok(app_context)
    }

    /// Construct an ApplicationContext from a path and root using default MainOptions.
    pub fn from_path_and_root(
        pathbuf: std::path::PathBuf,
        root: Option<&std::path::PathBuf>,
    ) -> Result<Self, errors::GardenError> {
        let options = cli::MainOptions::new();
        let app_context = Self::new(options.clone());
        let config_verbose = options.debug_level("config");
        app_context.get_root_config_mut().update(
            &app_context,
            Some(&pathbuf),
            root,
            config_verbose,
            None,
        )?;
        // Record the ID in the configuration.
        config::read_grafts(&app_context)?;

        Ok(app_context)
    }

    /// Construct an ApplicationContext from a path using default MainOptions.
    pub fn from_path(pathbuf: std::path::PathBuf) -> Result<Self, errors::GardenError> {
        if let Some(root_dir) = pathbuf.parent().map(std::path::Path::to_owned) {
            Self::from_path_and_root(pathbuf, Some(&root_dir))
        } else {
            Self::from_path_and_root(pathbuf, None)
        }
    }

    /// Construct an ApplicationContext from a path using default MainOptions.
    pub fn from_path_string(path: &str) -> Result<Self, errors::GardenError> {
        Self::from_path(std::path::PathBuf::from(path))
    }

    /// Construct an ApplicationContext from a string using default MainOptions.
    pub fn from_string(string: &str) -> Result<Self, errors::GardenError> {
        let options = cli::MainOptions::new();
        let app_context = Self::new(options);

        config::parse(&app_context, string, 0, app_context.get_root_config_mut())?;
        config::read_grafts(&app_context)?;

        Ok(app_context)
    }

    pub fn get_config(&self, id: ConfigId) -> &Configuration {
        unsafe { (*self.arena.as_ptr()).get(id).unwrap().get() }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn get_config_mut(&self, id: ConfigId) -> &mut Configuration {
        unsafe { (*self.arena.as_ptr()).get_mut(id).unwrap().get_mut() }
    }

    pub fn get_root_id(&self) -> ConfigId {
        self.root_id
    }

    pub fn get_root_config(&self) -> &Configuration {
        self.get_config(self.get_root_id())
    }

    pub fn get_root_config_mut(&self) -> &mut Configuration {
        self.get_config_mut(self.get_root_id())
    }

    /// Add a child Configuration graft onto the parent ConfigId.
    pub fn add_graft(&self, parent: ConfigId, config: Configuration) -> ConfigId {
        let graft_id = self.arena.borrow_mut().new_node(config); // Take ownership of config.
        parent.append(graft_id, &mut self.arena.borrow_mut());

        self.get_config_mut(graft_id).set_id(graft_id);

        graft_id
    }

    /// Attach a graft to the configuration specified by ConfigId.
    pub fn add_graft_config(
        &self,
        config_id: ConfigId,
        graft_name: &str,
        path: &std::path::Path,
        root: Option<&std::path::PathBuf>,
    ) -> Result<(), errors::GardenError> {
        let path = path.to_path_buf();
        let config_verbose = self.options.debug_level("config");
        let mut graft_config = Configuration::new();
        graft_config.update(self, Some(&path), root, config_verbose, Some(config_id))?;

        // The app Arena takes ownershp of the Configuration.
        let graft_id = self.add_graft(config_id, graft_config);
        // Record the graft's config ID in the graft.
        if let Some(graft_config) = self.get_config_mut(config_id).grafts.get_mut(graft_name) {
            graft_config.set_id(graft_id);
        }
        // Read child grafts recursively.
        config::read_grafts_recursive(self, graft_id)?;

        Ok(())
    }
}

/// Represent the different types of Git worktree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GitTreeType {
    Parent,           // A worktree with child worktrees attached to it.
    Worktree(String), // A child worktree created with "git worktree".
    Tree,             // A plain ole Git clone / worktree created with "git clone/init".
    Bare,             // A bare repository.
}

impl_display!(GitTreeType);

/// Represent "git worktree list" details queried from Git.
#[derive(Clone, Debug)]
pub struct GitTreeDetails {
    pub branch: String,
    pub tree_type: GitTreeType,
}

impl_display!(GitTreeDetails);
