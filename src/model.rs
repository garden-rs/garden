extern crate glob;

use super::eval;
use super::syntax;


/// Remotes at minimum have a name and a URL
#[derive(Clone, Debug)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

impl_display_brief!(Remote);


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
    pub expr: String,
    pub value: Option<String>,
}

impl_display_brief!(Variable);

// Named variables with a single value
#[derive(Clone, Debug)]
pub struct NamedVariable  {
    pub name: String,
    pub expr: String,
    pub value: Option<String>,
}

impl_display_brief!(NamedVariable);

// Simple Name/Value pairs
#[derive(Clone, Debug)]
pub struct NamedValue {
    pub name: String,
    pub value: String,
}

impl_display_brief!(NamedValue);

// Named variables with multiple values
#[derive(Clone, Debug)]
pub struct MultiVariable {
    pub name: String,
    pub values: Vec<Variable>,
}

impl_display!(MultiVariable);

// Trees represent a single worktree
#[derive(Clone, Debug, Default)]
pub struct Tree {
    pub name: String,
    pub path: Variable,
    pub templates: Vec<String>,
    pub remotes: Vec<Remote>,
    pub gitconfig: Vec<NamedVariable>,
    pub variables: Vec<NamedVariable>,
    pub environment: Vec<MultiVariable>,
    pub commands: Vec<MultiVariable>,
}

impl_display!(Tree);

impl Tree {
    pub fn reset_variables(&mut self) {
        // self.path is a variable but it is not reset because
        // the tree path is evaluated once when the configuration
        // is first read, and never again.
        for var in &mut self.variables {
            var.value = None;
        }

        for var in &mut self.gitconfig {
            var.value = None;
        }

        for env in &mut self.environment {
            for var in &mut env.values {
                var.value = None;
            }
        }
        for cmd in &mut self.commands {
            for var in &mut cmd.values {
                var.value = None;
            }
        }
    }
}


#[derive(Clone, Debug, Default)]
pub struct Group {
    pub name: String,
    pub members: Vec<String>,
}

impl_display!(Group);


#[derive(Clone, Debug, Default)]
pub struct Template {
    pub name: String,
    pub extend: Vec<String>,
    pub variables: Vec<NamedVariable>,
    pub environment: Vec<MultiVariable>,
    pub commands: Vec<MultiVariable>,
    pub gitconfig: Vec<NamedVariable>,
}

impl_display!(Template);


// Gardens aggregate trees
#[derive(Clone, Debug, Default)]
pub struct Garden {
    pub name: String,
    pub commands: Vec<MultiVariable>,
    pub environment: Vec<MultiVariable>,
    pub gitconfig: Vec<NamedVariable>,
    pub groups: Vec<String>,
    pub trees: Vec<String>,
    pub variables: Vec<NamedVariable>,
}

impl_display!(Garden);

// Configuration represents an instantiated garden configuration
#[derive(Clone, Debug, Default)]
pub struct Configuration {
    pub commands: Vec<MultiVariable>,
    pub debug: std::collections::HashSet<String>,
    pub environment: Vec<MultiVariable>,
    pub gardens: Vec<Garden>,
    pub groups: Vec<Group>,
    pub path: Option<std::path::PathBuf>,
    pub root: Variable,
    pub root_path: std::path::PathBuf,
    pub shell: String,
    pub templates: Vec<Template>,
    pub tree_search_path: Vec<std::path::PathBuf>,
    pub trees: Vec<Tree>,
    pub variables: Vec<NamedVariable>,
    pub verbose: bool,
}

impl_display!(Configuration);

impl Configuration {

    /// Create a default Configuration
    pub fn new() -> Self {
        return Configuration {
            shell: "zsh".to_string(),
            root: Variable {
                expr: "~/src".to_string(),
                value: None,
            },
            ..std::default::Default::default()
        }
    }

    pub fn initialize(&mut self) {
        // Evaluate garden.root
        let expr = self.root.expr.to_string();
        let value = eval::value(self, expr);
        // Store the resolved garden.root
        self.root_path = std::path::PathBuf::from(value.to_string());
        self.root.value = Some(value);

        // Resolve tree paths
        self.update_tree_paths();
    }

    // Calculate the "path" field for each tree.
    // If specified as a relative path, it will be relative to garden.root.
    // If specified as an asbolute path, it will be left as-is.
    pub fn update_tree_paths(&mut self) {
        let mut values = vec!();
        for tree in &self.trees {
            values.push(tree.path.expr.to_string());
        }

        for (idx, value) in values.iter().enumerate() {
            let result = eval::value(self, value.to_string());
            let tree = &mut self.trees[idx];

            if result.starts_with("/") {
                // Absolute path, nothing to do
                tree.path.value = Some(result);
            } else {
                // Make path relative to root_path
                let mut path_buf = self.root_path.to_path_buf();
                path_buf.push(result);
                tree.path.value = Some(path_buf.to_string_lossy().to_string());
            }
        }

        // Reset variables to allow for tree-scope evaluation
        self.reset_variables();

        let mut idx = 0;
        while idx < values.len() {
            let tree = &mut self.trees[idx];
            idx += 1;
            // ${TREE_PATH} is automatically available in each tree
            let tree_path = tree.path.value.as_ref().unwrap().to_string();
            tree.variables.push(
                NamedVariable {
                    name: "TREE_PATH".to_string(),
                    expr: tree_path.to_string(),
                    value: Some(tree_path.to_string()),
                }
            );

        }
    }

    /// Reset resolved variables
    pub fn reset_variables(&mut self) {
        for var in &mut self.variables {
            var.value = None;
        }
        for env in &mut self.environment {
            for var in &mut env.values {
                var.value = None;
            }
        }
        for cmd in &mut self.commands {
            for var in &mut cmd.values {
                var.value = None;
            }
        }
        for tree in &mut self.trees {
            tree.reset_variables();
        }
    }
}


/// Tree index into config.trees
pub type TreeIndex = usize;

/// Garden index into config.gardens
pub type GardenIndex = usize;


#[derive(Debug)]
pub struct TreeContext {
    pub tree: TreeIndex,
    pub garden: Option<GardenIndex>,
}

impl_display_brief!(TreeContext);


#[derive(Debug, Default)]
pub struct TreeExpression {
    pub pattern: glob::Pattern,
    pub is_default: bool,
    pub is_garden: bool,
    pub is_group: bool,
    pub is_tree: bool,
    pub include_gardens: bool,
    pub include_groups: bool,
    pub include_trees: bool,
}

impl_display_brief!(TreeExpression);

impl TreeExpression {

    pub fn new<S: Into<String>>(expr: S) -> Self {
        let mut glob_pattern = expr.into();
        let mut is_default = false;
        let mut is_tree = false;
        let mut is_garden = false;
        let mut is_group = false;
        let mut include_gardens = true;
        let mut include_groups = true;
        let mut include_trees = true;
        let mut trim = false;

        if syntax::is_garden(&glob_pattern) {
            is_garden = true;
            include_groups = false;
            include_trees = false;
            trim = true;
        } else if syntax::is_group(&glob_pattern) {
            is_group = true;
            include_gardens = false;
            include_trees = false;
            trim = true;
        } else if syntax::is_tree(&glob_pattern) {
            is_tree = true;
            include_gardens = false;
            include_groups = false;
            trim = true;
        } else {
            is_default = true;
        }
        if trim {
            glob_pattern = syntax::trim(&glob_pattern);
        }

        let pattern = glob::Pattern::new(glob_pattern.as_ref()).unwrap();

        return TreeExpression {
            is_default: is_default,
            is_garden: is_garden,
            is_group: is_group,
            is_tree: is_tree,
            include_gardens: include_gardens,
            include_groups: include_groups,
            include_trees: include_trees,
            pattern: pattern,
        }
    }
}
