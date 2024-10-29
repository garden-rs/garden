use std::cell::{Cell, RefCell, UnsafeCell};
use std::str::FromStr;

use derivative::Derivative;
use indextree::{Arena, NodeId};
use is_terminal::IsTerminal;
use strum::VariantNames;
use strum_macros;
use which::which;

use crate::{cli, collections, config, constants, errors, eval, path, syntax};

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V>;
pub(crate) type IndexSet<V> = indexmap::IndexSet<V>;
pub(crate) type StringSet = indexmap::IndexSet<String>;

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

/// Environment variables are set when running commands.
pub(crate) type Environment = Vec<(String, String)>;

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
#[derive(Debug, Default)]
pub struct Variable {
    expr: String,
    value: UnsafeCell<Option<String>>,
    evaluating: Cell<bool>,
}

impl_display_brief!(Variable);

/// A custom thread-safe clone implementation. RefCell::clone() is not thread-safe
/// because it mutably borrows data under the hood.
impl Clone for Variable {
    fn clone(&self) -> Self {
        Self {
            expr: self.expr.clone(),
            value: UnsafeCell::new(self.get_value().cloned()),
            evaluating: Cell::new(false),
        }
    }
}

impl Variable {
    pub(crate) fn new(expr: String, value: Option<String>) -> Self {
        Variable {
            expr,
            value: UnsafeCell::new(value),
            evaluating: Cell::new(false),
        }
    }

    /// Does this variable have a value?
    pub(crate) fn is_empty(&self) -> bool {
        self.expr.is_empty()
    }

    /// Is this variable currently being evaluated?
    /// This is a guard variable to avoid infinite loops when evaluating.
    pub(crate) fn is_evaluating(&self) -> bool {
        self.evaluating.get()
    }

    /// Set the evaluation state.
    pub(crate) fn set_evaluating(&self, value: bool) {
        self.evaluating.set(value);
    }

    /// Return the raw expression for this variable.
    pub fn get_expr(&self) -> &String {
        &self.expr
    }

    /// Return a mutable reference to the underlying raw expression.
    pub(crate) fn get_expr_mut(&mut self) -> &mut String {
        &mut self.expr
    }

    /// Set the expression for this variable.
    pub(crate) fn set_expr(&mut self, expr: String) {
        self.expr = expr;
    }

    /// Store the cached result of evaluating the expression.
    pub(crate) fn set_value(&self, value: String) {
        unsafe {
            *self.value.get() = Some(value);
        }
    }

    /// Transform the `RefCell<Option<String>>` value into `Option<&String>`.
    pub fn get_value(&self) -> Option<&String> {
        unsafe { (*self.value.get()).as_ref() }
    }

    /// Reset the variable.
    pub(crate) fn reset(&self) {
        unsafe {
            *self.value.get() = None;
        }
    }
}

/// An unordered mapping of names to a vector of Variables.
pub(crate) type MultiVariableMap = IndexMap<String, Vec<Variable>>;

/// Reset the variables held inside a MultiVariableMap.
fn reset_map_variables(vec_variables: &MultiVariableMap) {
    for variables in vec_variables.values() {
        for variable in variables {
            variable.reset();
        }
    }
}

/// An unordered mapping of name to Variable.
pub(crate) type VariableMap = IndexMap<String, Variable>;

// Named variables with multiple values
#[derive(Clone, Debug)]
pub struct MultiVariable {
    name: String,
    variables: Vec<Variable>,
}

impl_display!(MultiVariable);

impl MultiVariable {
    pub(crate) fn new(name: String, variables: Vec<Variable>) -> Self {
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
#[derive(Clone, Debug, Derivative)]
#[derivative(Default)]
pub struct Tree {
    pub commands: MultiVariableMap,
    pub environment: Vec<MultiVariable>,
    pub gitconfig: MultiVariableMap,
    pub remotes: VariableMap,
    pub(crate) symlink: Variable,
    pub templates: StringSet,
    pub variables: VariableMap,
    pub branch: Variable,
    pub(crate) branches: VariableMap,
    pub worktree: Variable,
    #[derivative(Default(value = r#""origin".to_string()"#))]
    pub(crate) default_remote: String,
    pub(crate) clone_depth: i64,
    pub(crate) is_single_branch: bool,
    pub is_symlink: bool,
    pub is_bare_repository: bool,
    pub is_worktree: bool,
    pub(crate) description: String,
    pub(crate) links: Vec<Variable>,

    name: String,
    path: Variable,
}

impl_display!(Tree);

impl Tree {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Set the tree name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub(crate) fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub fn get_path(&self) -> &Variable {
        &self.path
    }

    pub(crate) fn get_path_mut(&mut self) -> &mut Variable {
        &mut self.path
    }

    pub(crate) fn path_is_valid(&self) -> bool {
        self.path.get_value().is_some()
    }

    /// Build a canonicalized pathbuf for the current tree.
    pub(crate) fn canonical_pathbuf(&self) -> Option<std::path::PathBuf> {
        if let Some(pathbuf) = self.pathbuf() {
            if let Ok(canon_path) = pathbuf.canonicalize() {
                return Some(canon_path);
            }
        }

        None
    }

    /// Build a pathbuf for the current tree.
    pub(crate) fn pathbuf(&self) -> Option<std::path::PathBuf> {
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

    pub(crate) fn symlink_as_ref(&self) -> Result<&String, errors::GardenError> {
        match self.symlink.get_value() {
            Some(value) => Ok(value),
            None => Err(errors::GardenError::ConfigurationError(format!(
                "unset tree path for {}",
                self.name
            ))),
        }
    }

    /// Add the builtin TREE_NAME and TREE_PATH variables.
    pub(crate) fn add_builtin_variables(&mut self) {
        self.variables.insert(
            string!(constants::TREE_NAME),
            Variable::new(self.get_name().clone(), None),
        );

        // Register the ${TREE_PATH} variable.
        self.variables.insert(
            string!(constants::TREE_PATH),
            Variable::new(self.get_path().get_expr().clone(), None),
        );
    }

    pub(crate) fn reset_variables(&self) {
        // self.path is a variable but it is not reset because
        // the tree path is evaluated once when the configuration
        // is first read, and never again.
        for var in self.variables.values() {
            var.reset();
        }
        for env in &self.environment {
            env.reset();
        }

        reset_map_variables(&self.gitconfig);
        reset_map_variables(&self.commands);
    }

    /// Copy the guts of another tree into the current tree.
    pub(crate) fn clone_from_tree(&mut self, tree: &Tree) {
        collections::append_map(&mut self.commands, &tree.commands);
        collections::append_map(&mut self.gitconfig, &tree.gitconfig);
        collections::append_map(&mut self.variables, &tree.variables);
        collections::append_map(&mut self.remotes, &tree.remotes);
        collections::append_set(&mut self.templates, &tree.templates);

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
        self.default_remote = tree.default_remote.to_string();
        self.description = tree.description.to_string();
        self.links.clone_from(&tree.links);

        self.update_flags();
    }

    /// Update internal flags in response to newly read data.
    pub(crate) fn update_flags(&mut self) {
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

    /// Return the resolved "branch" field.
    pub(crate) fn eval_branch(&self, eval_context: &EvalContext) -> String {
        self.get_branch(
            eval_context.app_context,
            eval_context.config,
            eval_context.graft_config,
            eval_context.tree_context,
        )
    }

    /// Return the resolved "branch" field.
    pub(crate) fn get_branch(
        &self,
        app_context: &ApplicationContext,
        config: &Configuration,
        graft_config: Option<&Configuration>,
        tree_context: &TreeContext,
    ) -> String {
        eval::tree_variable(
            app_context,
            config,
            graft_config,
            &tree_context.tree,
            tree_context.garden.as_ref(),
            &self.branch,
        )
    }

    // Return the resolved "url" field for the default remote.
    pub(crate) fn eval_url(&self, eval_context: &EvalContext) -> Option<String> {
        self.get_url(
            eval_context.app_context,
            eval_context.config,
            eval_context.graft_config,
            eval_context.tree_context,
        )
    }

    // Return the resolved "url" field for the default remote.
    pub(crate) fn get_url(
        &self,
        app_context: &ApplicationContext,
        config: &Configuration,
        graft_config: Option<&Configuration>,
        context: &TreeContext,
    ) -> Option<String> {
        self.remotes.get(&self.default_remote).map(|remote| {
            eval::tree_variable(
                app_context,
                config,
                graft_config,
                &context.tree,
                context.garden.as_ref(),
                remote,
            )
        })
    }

    /// Return the resolved "worktree" field.
    pub(crate) fn eval_worktree(&self, eval_context: &EvalContext) -> String {
        self.get_worktree(
            eval_context.app_context,
            eval_context.config,
            eval_context.graft_config,
            eval_context.tree_context,
        )
    }

    /// Return the resolved "worktree" field.
    pub(crate) fn get_worktree(
        &self,
        app_context: &ApplicationContext,
        config: &Configuration,
        graft_config: Option<&Configuration>,
        tree_context: &TreeContext,
    ) -> String {
        eval::tree_variable(
            app_context,
            config,
            graft_config,
            &tree_context.tree,
            tree_context.garden.as_ref(),
            &self.worktree,
        )
    }

    /// Return the remote associated with a branch to checkout.
    pub(crate) fn get_remote_for_branch(
        &self,
        eval_context: &EvalContext,
        branch: &str,
    ) -> Option<String> {
        let remote_branch = self.get_upstream_branch(eval_context, branch)?;
        let remote = remote_branch.split_once('/')?.0;
        if self.remotes.contains_key(remote) {
            Some(remote.to_string())
        } else {
            None
        }
    }

    /// Return the remote branch associated with a local branch.
    pub(crate) fn get_upstream_branch(
        &self,
        eval_context: &EvalContext,
        branch: &str,
    ) -> Option<String> {
        if branch.is_empty() {
            return None;
        }
        self.branches
            .get(branch)
            .map(|remote_branch_var| eval_context.tree_variable(remote_branch_var))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Group {
    name: String,
    pub members: StringSet,
}

impl_display!(Group);

impl Group {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Return an owned copy of the name field.
    pub(crate) fn get_name_owned(&self) -> String {
        self.get_name().to_owned()
    }

    pub(crate) fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }
}

/// Groups are stored in a GroupMap inside Configuration.
pub type GroupMap = IndexMap<GroupName, Group>;

/// Templates can be used to create trees.
/// They contain a (path-less) tree object which can be used for creating
/// materialized trees.
#[derive(Clone, Debug, Default)]
pub struct Template {
    pub tree: Tree,
    pub extend: StringSet,
    name: String,
}

impl_display!(Template);

impl Template {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub(crate) fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    /// Apply this template onto the specified tree.
    pub(crate) fn apply(&self, tree: &mut Tree) {
        tree.clone_from_tree(&self.tree);
    }
}

// Gardens aggregate trees
#[derive(Clone, Debug, Default)]
pub struct Garden {
    pub commands: MultiVariableMap,
    pub environment: Vec<MultiVariable>,
    pub gitconfig: MultiVariableMap,
    pub groups: StringSet,
    pub trees: StringSet,
    pub variables: VariableMap,
    name: GardenName,
}

impl_display!(Garden);

impl Garden {
    pub fn get_name(&self) -> &GardenName {
        &self.name
    }

    pub(crate) fn get_name_mut(&mut self) -> &mut String {
        &mut self.name
    }
}

/// Gardens are stored in a GardenMap inside Configuration.
pub type GardenMap = IndexMap<GardenName, Garden>;

/// Return the default shell to use for custom commands and "garden shell".
fn get_default_shell() -> String {
    if which(constants::SHELL_ZSH).is_ok() {
        constants::SHELL_ZSH
    } else if which(constants::SHELL_BASH).is_ok() {
        constants::SHELL_BASH
    } else if which(constants::SHELL_DASH).is_ok() {
        constants::SHELL_DASH
    } else {
        constants::SHELL_SH
    }
    .to_string()
}

/// Configuration represents an instantiated garden configuration
#[derive(Clone, Debug, Default)]
pub struct Configuration {
    pub commands: MultiVariableMap,
    pub debug: IndexMap<String, u8>,
    pub environment: Vec<MultiVariable>,
    pub gardens: GardenMap,
    pub grafts: IndexMap<GraftName, Graft>,
    pub groups: GroupMap,
    pub path: Option<std::path::PathBuf>,
    pub dirname: Option<std::path::PathBuf>,
    pub root: Variable,
    pub root_is_dynamic: bool,
    pub root_path: std::path::PathBuf,
    pub shell: String,
    pub interactive_shell: String,
    pub templates: IndexMap<String, Template>,
    pub tree_search_path: Vec<std::path::PathBuf>,
    pub trees: IndexMap<TreeName, Tree>,
    pub variables: VariableMap,
    /// Variables defined on the command-line using "-D name=value" have the
    /// highest precedence and override variables defined by any configuration or tree.
    pub override_variables: VariableMap,
    pub config_verbose: u8,
    pub quiet: bool,
    pub verbose: u8,
    pub(crate) shell_exit_on_error: bool,
    pub(crate) shell_word_split: bool,
    pub(crate) tree_branches: bool,
    pub(crate) parent_id: Option<ConfigId>,
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
            shell_exit_on_error: true,
            shell_word_split: true,
            tree_branches: true,
            ..std::default::Default::default()
        }
    }

    pub(crate) fn initialize(&mut self, app_context: &ApplicationContext) {
        // Evaluate garden.root
        let expr = self.root.get_expr().to_string();
        let mut value = eval::value(app_context, self, &expr);
        if expr.is_empty() {
            if self.root_is_dynamic {
                // Default to the current directory when garden.root is configured to
                // the empty string.
                let current_dir = path::current_dir_string();
                self.root.set_expr(current_dir.clone());
                self.root.set_value(current_dir);
                self.root_path = path::current_dir();
            } else {
                // Default garden.root to ${GARDEN_CONFIG_DIR} by default.
                self.root
                    .set_expr(string!(constants::GARDEN_CONFIG_DIR_EXPR));
                if let Some(ref dirname) = self.dirname {
                    self.root.set_value(dirname.to_string_lossy().to_string());
                    self.root_path = dirname.to_path_buf();
                }
            }
        } else {
            // Store the resolved, canonicalized garden.root
            self.root_path = std::path::PathBuf::from(&value);
            if let Ok(root_path_canon) = self.root_path.canonicalize() {
                if root_path_canon != self.root_path {
                    value = root_path_canon
                        .to_str()
                        .unwrap_or(value.as_str())
                        .to_string();
                    self.root_path = root_path_canon;
                }
            }
            self.root.set_value(value);
        }
        self.update_tree_paths(app_context); // Resolve tree paths
        self.synthesize_default_tree(); // Synthesize a tree if no trees exist.
                                        // Reset variables
        self.reset();
    }

    /// Return Some(&NodeId) when the configuration is a graft and None otherwise.
    pub(crate) fn graft_id(&self) -> Option<NodeId> {
        self.parent_id.and(self.get_id())
    }

    pub(crate) fn update(
        &mut self,
        app_context: &ApplicationContext,
        config: Option<&std::path::Path>,
        root: Option<&std::path::Path>,
        config_verbose: u8,
        parent: Option<ConfigId>,
    ) -> Result<(), errors::GardenError> {
        if let Some(parent_id) = parent {
            self.set_parent(parent_id);
        }
        self.config_verbose = config_verbose;
        self.quiet = app_context.options.quiet;
        self.verbose = app_context.options.verbose;

        // Override the configured garden root
        let root_pathbuf_option =
            root.map(|path| path.canonicalize().unwrap_or(path.to_path_buf()));
        if let Some(root_path) = root_pathbuf_option {
            self.root.set_expr(root_path.to_string_lossy().to_string());
        }

        let mut basename = string!(constants::GARDEN_CONFIG);

        // Find garden.yaml in the search path
        let mut found = false;
        if let Some(config_path) = config {
            if config_path.is_file() || config_path.is_absolute() {
                // If an absolute path was specified, or if the file exists,
                // short-circuit the search; the config file might be missing but
                // we shouldn't silently use a different config file.
                self.set_path(&config_path);
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
                    self.set_path(&candidate);
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
            }
        }

        Ok(())
    }

    /// Apply MainOptions to a Configuration.
    pub(crate) fn update_options(
        &mut self,
        options: &cli::MainOptions,
    ) -> Result<(), errors::GardenError> {
        let config_verbose = options.debug_level(constants::DEBUG_LEVEL_CONFIG);
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
        self.apply_defines(&options.define);

        Ok(())
    }

    // Apply --define name=value options.
    pub(crate) fn apply_defines(&mut self, defines: &Vec<String>) {
        for k_eq_v in defines {
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
            // Allow overridding garden.<value> using "garden -D garden.<value>=false".
            match name.as_str() {
                constants::GARDEN_INTERACTIVE_SHELL => {
                    self.interactive_shell = expr;
                }
                constants::GARDEN_SHELL => {
                    self.shell = expr;
                }
                constants::GARDEN_SHELL_ERREXIT => {
                    set_bool(name.as_str(), &expr, &mut self.shell_exit_on_error);
                }
                constants::GARDEN_SHELL_WORDSPLIT => {
                    set_bool(name.as_str(), &expr, &mut self.shell_word_split);
                }
                constants::GARDEN_TREE_BRANCHES => {
                    set_bool(name.as_str(), &expr, &mut self.tree_branches);
                }
                _ => {
                    self.override_variables
                        .insert(name, Variable::new(expr, None));
                }
            }
        }
    }

    /// Apply the quiet and verbose options to resolve GARDEN_CMD_VERBOSE and GARDEN_CMD_QUIET.
    pub(crate) fn update_quiet_and_verbose_variables(&mut self, quiet: bool, verbose: u8) {
        // Provide GARDEN_CMD_QUIET and GARDEN_CMD_VERBOSE.
        let quiet_string = if self.quiet || quiet { "--quiet" } else { "" }.to_string();
        self.variables.insert(
            string!(constants::GARDEN_CMD_QUIET),
            Variable::new(quiet_string.clone(), Some(quiet_string)),
        );
        let verbose = self.verbose + verbose;
        let verbose_string = if verbose > 0 {
            format!("-{}", "v".repeat(verbose.into()))
        } else {
            string!("")
        };
        self.variables.insert(
            string!(constants::GARDEN_CMD_VERBOSE),
            Variable::new(verbose_string.clone(), Some(verbose_string)),
        );
    }

    pub(crate) fn reset(&mut self) {
        // Reset variables to allow for tree-scope evaluation
        self.reset_variables();
        // Add custom variables
        self.reset_builtin_variables()
    }

    fn reset_builtin_variables(&mut self) {
        // Update GARDEN_ROOT.
        if let Some(var) = self.variables.get_mut(constants::GARDEN_ROOT) {
            if let Some(value) = self.root.get_value() {
                var.set_expr(value.into());
                var.set_value(value.into());
            }
        }

        for tree in self.trees.values_mut() {
            // Update TREE_NAME.
            let tree_name = String::from(tree.get_name());
            if let Some(var) = tree.variables.get_mut(constants::TREE_NAME) {
                var.set_expr(tree_name.to_string());
                var.set_value(tree_name);
            }
            // Extract the tree's path.  Skip invalid/unset entries.
            let tree_path = match tree.path_as_ref() {
                Ok(path) => String::from(path),
                Err(_) => continue,
            };
            // Update TREE_PATH.
            if let Some(var) = tree.variables.get_mut(constants::TREE_PATH) {
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

    /// Create an implicit "." tree when no trees exist.
    fn synthesize_default_tree(&mut self) {
        if !self.commands.is_empty() && self.trees.is_empty() {
            let dirname_string = self.dirname_string();
            let mut tree = Tree::default();
            tree.path.set_expr(dirname_string.clone());
            tree.path.set_value(dirname_string);
            tree.description = string!("The default tree for garden commands.");
            tree.add_builtin_variables();
            tree.set_name(string!(constants::DOT));
            self.trees.insert(string!(constants::DOT), tree);
        }
    }

    /// Return a path string relative to the garden root
    pub(crate) fn tree_path(&self, path: &str) -> String {
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
    pub(crate) fn relative_pathbuf(&self, path: &str) -> std::path::PathBuf {
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
    fn eval_tree_path(&mut self, app_context: &ApplicationContext, path: &str) -> String {
        let value = eval::value(app_context, self, path);
        self.tree_path(&value)
    }

    /// Resolve a pathbuf relative to the config directory.
    pub(crate) fn config_pathbuf(&self, path: &str) -> Option<std::path::PathBuf> {
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
    fn config_pathbuf_from_include(
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

        // Make path relative to the configuration's dirname.
        self.config_pathbuf(path)
    }

    /// Resolve a path string relative to the config directory.
    fn config_path(&self, path: &str) -> String {
        if let Some(path_buf) = self.config_pathbuf(path) {
            path_buf.to_string_lossy().to_string()
        } else {
            self.tree_path(path)
        }
    }

    /// Return a directory string for the garden config directory.
    /// This typically returns GARDEN_CONFIG_DIR but falls back to
    /// the current directory when unset.
    fn dirname_string(&self) -> String {
        match self.dirname {
            Some(ref dirname) => dirname.to_string_lossy().to_string(),
            None => path::current_dir_string(),
        }
    }

    /// Return a path for running commands that should always exist.
    pub(crate) fn fallback_execdir_string(&self) -> String {
        if self.root_path.exists() {
            return self.root_path.to_string_lossy().to_string();
        }
        if let Some(dirname) = self.dirname.as_ref() {
            if dirname.exists() {
                return dirname.to_string_lossy().to_string();
            }
        }
        path::current_dir_string()
    }

    /// Evaluate and resolve a path string and relative to the config directory.
    pub(crate) fn eval_config_path(&self, app_context: &ApplicationContext, path: &str) -> String {
        let value = eval::value(app_context, self, path);
        self.config_path(&value)
    }

    /// Evaluate and resolve a pathbuf relative to the config directory for "includes".
    pub(crate) fn eval_config_pathbuf_from_include(
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
    pub(crate) fn reset_variables(&mut self) {
        for var in self.variables.values() {
            var.reset();
        }
        for env in &self.environment {
            env.reset();
        }

        reset_map_variables(&self.commands);

        for tree in self.trees.values() {
            tree.reset_variables();
        }
    }

    /// Set the ConfigId from the Arena for this configuration.
    pub(crate) fn set_id(&mut self, id: ConfigId) {
        self.id = Some(id);
    }

    pub(crate) fn get_id(&self) -> Option<ConfigId> {
        self.id
    }

    /// Set the parent ConfigId from the Arena for this configuration.
    pub(crate) fn set_parent(&mut self, id: ConfigId) {
        self.parent_id = Some(id);
    }

    /// Set the config path and the dirname fields
    pub(crate) fn set_path(&mut self, path: &dyn AsRef<std::path::Path>) {
        let config_path = path.as_ref().to_path_buf();
        let mut dirname = config_path.clone();
        dirname.pop();

        self.dirname = Some(dirname);
        self.path = Some(config_path);
    }

    /// Get the config path if it is defined.
    pub(crate) fn get_path(&self) -> Result<&std::path::PathBuf, errors::GardenError> {
        self.path.as_ref().ok_or_else(|| {
            errors::GardenError::AssertionError("Configuration path is unset".into())
        })
    }

    /// Get a path string for this configuration.
    /// Returns the current directory when the configuration does not have a valid path.
    pub(crate) fn get_path_for_display(&self) -> String {
        let default_pathbuf = std::path::PathBuf::from(constants::DOT);
        self.path
            .as_ref()
            .unwrap_or(&default_pathbuf)
            .display()
            .to_string()
    }

    /// Return true if the configuration contains the named graft.
    pub(crate) fn contains_graft(&self, name: &str) -> bool {
        let graft_name = syntax::trim(name);
        self.grafts.contains_key(graft_name)
    }

    /// Return a graft by name.
    pub(crate) fn get_graft(&self, name: &str) -> Result<&Graft, errors::GardenError> {
        let graft_name = syntax::trim(name);
        self.grafts.get(graft_name).ok_or_else(|| {
            errors::GardenError::ConfigurationError(format!("{name}: no such graft"))
        })
    }

    /// Parse a "graft::value" string and return the ConfigId for the graft and the
    /// remaining unparsed "value".
    pub(crate) fn get_graft_id<'a>(
        &self,
        value: &'a str,
    ) -> Result<(ConfigId, &'a str), errors::GardenError> {
        let (graft_name, remainder) = match syntax::split_graft(value) {
            Some((graft_name, remainder)) => (graft_name, remainder),
            None => {
                return Err(errors::GardenError::ConfigurationError(format!(
                    "{value}: invalid graft expression"
                )))
            }
        };
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
    pub(crate) fn get_tree_pathbuf(&self, tree_name: &str) -> Option<std::path::PathBuf> {
        self.get_tree(tree_name)
            .map(|tree| tree.canonical_pathbuf())
            .unwrap_or(None)
    }
}

/// Parse a named boolean value into a bool warn if the value is not a valid bool value.
fn set_bool(name: &str, expr: &str, output: &mut bool) {
    if let Some(value) = syntax::string_to_bool(expr) {
        *output = value;
    } else {
        error!(
            "'{}' is not a valid value for \"{}\". Must be true, false, 0 or 1",
            name, expr
        );
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

    pub(crate) fn set_id(&mut self, id: ConfigId) {
        self.id = Some(id);
    }
}

// TODO EvalContext
#[derive(Clone, Debug)]
pub(crate) struct EvalContext<'a> {
    pub(crate) app_context: &'a ApplicationContext,
    pub(crate) config: &'a Configuration,
    pub(crate) graft_config: Option<&'a Configuration>,
    pub(crate) tree_context: &'a TreeContext,
}

impl EvalContext<'_> {
    /// Construct a new EvalContext.
    pub(crate) fn new<'a>(
        app_context: &'a ApplicationContext,
        config: &'a Configuration,
        graft_config: Option<&'a Configuration>,
        tree_context: &'a TreeContext,
    ) -> EvalContext<'a> {
        EvalContext {
            app_context,
            config,
            graft_config,
            tree_context,
        }
    }

    /// Create an EvalContext from an ApplicationContext and TreeContext.
    pub(crate) fn from_app_context<'a>(
        app_context: &'a ApplicationContext,
        tree_context: &'a TreeContext,
    ) -> EvalContext<'a> {
        let config = app_context.get_root_config();
        let graft_config = tree_context
            .config
            .map(|config_id| app_context.get_config(config_id));
        EvalContext::new(app_context, config, graft_config, tree_context)
    }

    /// Evaluate a tree variable.
    pub(crate) fn tree_value(&self, value: &str) -> String {
        eval::tree_value(
            self.app_context,
            self.config,
            self.graft_config,
            value,
            &self.tree_context.tree,
            self.tree_context.garden.as_ref(),
        )
    }

    /// Evaluate a Variable with a tree scope.
    pub(crate) fn tree_variable(&self, var: &Variable) -> String {
        eval::tree_variable(
            self.app_context,
            self.config,
            self.graft_config,
            &self.tree_context.tree,
            self.tree_context.garden.as_ref(),
            var,
        )
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

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    strum_macros::EnumString,
    strum_macros::Display,
    strum_macros::VariantNames,
)]
#[strum(ascii_case_insensitive, serialize_all = "kebab-case")]
pub enum ColorMode {
    /// Enable color when a tty is detected
    #[default]
    Auto,
    /// Enable color
    #[strum(
        serialize = "1",
        serialize = "on",
        serialize = "true",
        serialize = "always",
        serialize = "y",
        serialize = "yes"
    )]
    On,
    /// Disable color
    #[strum(
        serialize = "0",
        serialize = "off",
        serialize = "false",
        serialize = "never",
        serialize = "n",
        serialize = "no"
    )]
    Off,
}

impl ColorMode {
    /// Parse a color mode from a string using strum's from_str().
    pub fn parse_from_str(string: &str) -> Result<ColorMode, String> {
        ColorMode::from_str(string).map_err(|_| format!("choices are {:?}", Self::VARIANTS))
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            ColorMode::Auto => std::io::stdout().is_terminal(),
            ColorMode::Off => false,
            ColorMode::On => true,
        }
    }

    pub(crate) fn update(&mut self) {
        if *self == ColorMode::Auto {
            // Speedup future calls to is_enabled() by performing the "auto"
            // is_terminal() check once and caching the result.
            if self.is_enabled() {
                *self = ColorMode::On;
            } else {
                *self = ColorMode::Off;
            }
        }

        if *self == ColorMode::Off {
            yansi::disable();
        }
    }
}

#[derive(Debug)]
pub struct ApplicationContext {
    pub options: cli::MainOptions,
    arena: RefCell<Arena<Configuration>>,
    root_id: ConfigId,
}

impl_display!(ApplicationContext);

/// Safety: ApplicationContext is not thread-safe due to its use of internal mutability.
/// Furthermore, we cannot use mutexes internally due to our use of Rayon and
/// [fundamental issues](https://github.com/rayon-rs/rayon/issues/592).
/// that lead to deadlocks when performing parallel computations.
///
/// ApplicationContext is designed to be cloned whenever operations are performed
/// across multiple threads. The RefCells stored inside of the Variable struct are
/// used to provider interior mutability. Even though the methods are const, the
/// methods mutate interior data in a non-thread-safe manner.
///
/// Likewise, even if we were able to hold mutexes when using Rayon, our access pattern
/// is not logically sound were we to attempt to evaluate data in parallel. The evaluation
/// machinery is context-specific and can lead to different results dependending on which
/// TreeContext initiated the eval. This is the core reason why cloning is required in
/// order to evaluate variables across multiple threads.
unsafe impl Sync for ApplicationContext {}

/// ApplicationContext performs a deepcopy of its internal arena to ensure that
/// none of its internally-mutable data is shared between threads.
impl Clone for ApplicationContext {
    fn clone(&self) -> Self {
        let mut arena: Arena<Configuration> = Arena::new();
        for node in self.arena.borrow().iter() {
            arena.new_node(node.get().clone());
        }
        Self {
            arena: RefCell::new(arena),
            options: self.options.clone(),
            root_id: self.root_id,
        }
    }
}

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
        let config_verbose = options.debug_level(constants::DEBUG_LEVEL_CONFIG);

        app_context.get_root_config_mut().update(
            &app_context,
            options.config.as_deref(),
            options.root.as_deref(),
            config_verbose,
            None,
        )?;
        app_context.get_root_config_mut().update_options(options)?;
        config::read_grafts(&app_context)?;

        Ok(app_context)
    }

    /// Construct an ApplicationContext from a path and root using default MainOptions.
    pub fn from_path_and_root(
        path: &dyn AsRef<std::path::Path>,
        root: Option<&std::path::Path>,
    ) -> Result<Self, errors::GardenError> {
        let options = cli::MainOptions::new();
        let app_context = Self::new(options.clone());
        let config_verbose = options.debug_level(constants::DEBUG_LEVEL_CONFIG);
        app_context.get_root_config_mut().update(
            &app_context,
            Some(path.as_ref()),
            root,
            config_verbose,
            None,
        )?;
        // Record the ID in the configuration.
        config::read_grafts(&app_context)?;

        Ok(app_context)
    }

    /// Construct an ApplicationContext from a path using default MainOptions.
    pub fn from_path(path: &dyn AsRef<std::path::Path>) -> Result<Self, errors::GardenError> {
        if let Some(root_dir) = path.as_ref().parent().map(std::path::Path::to_owned) {
            Self::from_path_and_root(path, Some(&root_dir))
        } else {
            Self::from_path_and_root(path, None)
        }
    }

    /// Construct an ApplicationContext from a path using default MainOptions.
    pub fn from_path_string(path: &str) -> Result<Self, errors::GardenError> {
        Self::from_path(&std::path::PathBuf::from(path))
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
    pub(crate) fn get_config_mut(&self, id: ConfigId) -> &mut Configuration {
        unsafe { (*self.arena.as_ptr()).get_mut(id).unwrap().get_mut() }
    }

    pub fn get_root_id(&self) -> ConfigId {
        self.root_id
    }

    pub fn get_root_config(&self) -> &Configuration {
        self.get_config(self.get_root_id())
    }

    pub(crate) fn get_root_config_mut(&self) -> &mut Configuration {
        self.get_config_mut(self.get_root_id())
    }

    /// Add a child Configuration graft onto the parent ConfigId.
    pub(crate) fn add_graft(&self, parent: ConfigId, config: Configuration) -> ConfigId {
        let graft_id = self.arena.borrow_mut().new_node(config); // Take ownership of config.
        parent.append(graft_id, &mut self.arena.borrow_mut());

        self.get_config_mut(graft_id).set_id(graft_id);

        graft_id
    }

    /// Attach a graft to the configuration specified by ConfigId.
    pub(crate) fn add_graft_config(
        &self,
        config_id: ConfigId,
        graft_name: &str,
        path: &std::path::Path,
        root: Option<&std::path::Path>,
    ) -> Result<(), errors::GardenError> {
        let path = path.to_path_buf();
        let config_verbose = self.options.debug_level(constants::DEBUG_LEVEL_CONFIG);
        let mut graft_config = Configuration::new();
        // Propagate the current config's settings onto child grafts.
        graft_config.tree_branches = self.get_config(config_id).tree_branches;
        graft_config.shell_exit_on_error = self.get_config(config_id).shell_exit_on_error;
        graft_config.shell_word_split = self.get_config(config_id).shell_word_split;
        // Parse the config file for the graft.
        graft_config.update(self, Some(&path), root, config_verbose, Some(config_id))?;

        // The app Arena takes ownership of the Configuration.
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
    /// A worktree with child worktrees attached to it.
    Parent,
    /// A child worktree created with "git worktree".
    Worktree(std::path::PathBuf),
    /// A regular Git clone / worktree created with "git clone/init".
    Tree,
    /// A bare repository.
    Bare,
}

impl_display!(GitTreeType);

/// Represent "git worktree list" details queried from Git.
#[derive(Clone, Debug)]
pub struct GitTreeDetails {
    pub branch: String,
    pub tree_type: GitTreeType,
}

impl_display!(GitTreeDetails);
