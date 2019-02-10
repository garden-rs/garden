extern crate glob;

use super::syntax;


// Remotes an minimum have a name and a URL
#[derive(Debug)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

impl_display_brief!(Remote);

/* Config files can define a sequence of variables that are
 * iteratively calculated.  Variables can reference other
 * variables in their Tree, Garden, and Configuration scopes.
 *
 * The config values can contain either plain values,
 * string ${expressions} that resolve against other Variables,
 * or exec expressions that evaluate to a command whose stdout is
 * captured and placed into the value of the variable.
 *
 * An exec expression can use shell-like ${variable} references as which
 * are substituted when evaluating the command, just like a regular
 * string expression.  An exec expression is denoted by using a "$ "
 * (dollar-sign followed by space) before the value.  For example,
 * using "$ echo foo" will place the value "foo" in the variable.
 */
#[derive(Debug)]
pub struct Variable {
    pub expr: String,
    pub value: Option<String>,
}

impl_display_brief!(Variable);

// Named variables with a single value
#[derive(Debug)]
pub struct NamedVariable  {
    pub name: String,
    pub expr: String,
    pub value: Option<String>,
}

impl_display_brief!(NamedVariable);

// Simple Name/Value pairs
#[derive(Debug)]
pub struct NamedValue {
    pub name: String,
    pub value: String,
}

impl_display_brief!(NamedValue);

// Named variables with multiple values
#[derive(Debug)]
pub struct MultiVariable {
    pub name: String,
    pub values: Vec<Variable>,
}

impl_display!(MultiVariable);

// Trees represent a single worktree
#[derive(Debug, Default)]
pub struct Tree {
    pub name: String,
    pub path: String,
    pub templates: Vec<String>,
    pub remotes: Vec<Remote>,
    pub gitconfig: Vec<NamedVariable>,
    pub variables: Vec<NamedVariable>,
    pub environment: Vec<MultiVariable>,
    pub commands: Vec<MultiVariable>,
}

impl_display!(Tree);

#[derive(Debug, Default)]
pub struct Group {
    pub name: String,
    pub members: Vec<String>,
}

impl_display!(Group);


#[derive(Debug, Default)]
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
#[derive(Debug, Default)]
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
#[derive(Debug, Default)]
pub struct Configuration {
    pub commands: Vec<MultiVariable>,
    pub debug: std::collections::HashSet<String>,
    pub environment: Vec<MultiVariable>,
    pub environment_variables: bool,
    pub gardens: Vec<Garden>,
    pub groups: Vec<Group>,
    pub path: Option<std::path::PathBuf>,
    pub root_path: std::path::PathBuf,
    pub shell: std::path::PathBuf,
    pub templates: Vec<Template>,
    pub tree_search_path: Vec<std::path::PathBuf>,
    pub trees: Vec<Tree>,
    pub variables: Vec<NamedVariable>,
    pub verbose: bool,
}


impl_display!(Configuration);


/// Create a default Configuration
impl Configuration {
    pub fn new() -> Self {
        return Configuration {
            environment_variables: true,
            shell: std::path::PathBuf::from("zsh"),
            ..std::default::Default::default()
        }
    }
}


/// Tree index into config.trees
pub type TreeIndex = u64;

/// Garden index into config.gardens
pub type GardenIndex = u64;


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
            glob_pattern.remove(0);
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
