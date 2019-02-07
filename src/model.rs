use std::collections::HashSet;

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
    pub debug: HashSet<String>,
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
