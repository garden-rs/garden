macro_rules! make_display {
    ($x:ident) => (
        impl std::fmt::Display for $x {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                return write!(f, "{:#?}", self);
            }
        }
    )
}

// Remotes an minimum have a name and a URL
#[derive(Debug)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

make_display!(Remote);

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

make_display!(Variable);

// Named variables with a single value
#[derive(Debug)]
pub struct NamedVariable  {
    pub name: String,
    pub var: Variable,
}
make_display!(NamedVariable);

// Simple Name/Value pairs
#[derive(Debug)]
pub struct NamedValue {
    pub name: String,
    pub value: String,
}

make_display!(NamedValue);

// Named variables with multiple values
#[derive(Debug)]
pub struct MultiVariable {
    pub name: String,
    pub values: Vec<Variable>,
}

make_display!(MultiVariable);

// Trees represent a single worktree
#[derive(Debug)]
pub struct Tree {
    pub name: String,
    pub path: std::path::PathBuf,
    pub remotes: Vec<Remote>,
    pub variables: Vec<Variable>,
    pub environment: Vec<MultiVariable>,
    pub commands: Vec<MultiVariable>,
    pub templates: Vec<String>,
    pub gitconfig: Vec<NamedValue>,
}

make_display!(Tree);

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub members: Vec<String>,
}

make_display!(Group);

// Gardens aggregate trees
#[derive(Debug)]
pub struct Garden {
    pub name: String,
    pub variables: Vec<NamedVariable>,
    pub templates: Vec<Tree>,
    pub trees: Vec<Tree>,
    pub environment: Vec<MultiVariable>,
    pub commands: Vec<MultiVariable>,
    pub gitconfig: Vec<NamedValue>,
}

make_display!(Garden);

// Configuration represents an instantiated garden configuration
#[derive(Debug)]
pub struct Configuration {
    pub path: Option<std::path::PathBuf>,
    pub variables: Vec<NamedVariable>,
    pub shell: std::path::PathBuf,
    pub environment: Vec<MultiVariable>,
    pub commands: Vec<MultiVariable>,
    pub tree_search_path: Vec<std::path::PathBuf>,
    pub root_path: std::path::PathBuf,
    pub gardens: Vec<Garden>,
    pub groups: Vec<String>,
    pub trees: Vec<Tree>,
    pub verbose: bool,
}

make_display!(Configuration);
