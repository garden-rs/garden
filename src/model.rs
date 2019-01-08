// Remotes an minimum have a name and a URL
struct Remote {
    name: String,
    url: String,
}

// Custom per-garden or per-tree Git configuration
pub struct NameValue {
    name: String,
    value: String,
}

// Config files can define a sequence of variables that are
// iteratively calculated.  Variables can reference other
// variables in their Tree, Garden, and Configuration scopes.
//
// The config file entries can have either plain values,
// "expr" string ${expressions} that resolve against other Variables,
// and exec statements that evaluate to the stdout of a subprocess.
pub struct Variable {
    name: String,
    value: Option<String>,
    exec: Option<String>,
    expr: Option<String>,
}

// Trees have many remotes
pub struct Tree {
    name: String,
    path: std::path::PathBuf,
    remotes: Vec<Remote>,
    variables: Vec<Variable>,
    environ: Vec<NameValue>,
    gitconfig: Vec<NameValue>,
}

// Gardens aggregate trees
pub struct Garden {
    name: String,
    trees: Vec<Tree>,
    variables: Vec<NameValue>,
    environ: Vec<NameValue>,
    gitconfig: Vec<NameValue>,
}
