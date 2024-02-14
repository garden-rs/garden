/// The "bare" key in a tree block defines a bare repository.
pub const BARE: &str = "bare";

/// The "branch" key in a tree block defines the branch to checkout on clone.
pub const BRANCH: &str = "branch";

/// The "branches" section in a tree block maps local to remote branches.
pub const BRANCHES: &str = "branches";

/// The "config" key in a graft definition defines the path to a garden config file.
pub const CONFIG: &str = "config";

/// The "commands" section in a configuration block defines custom commands.
pub const COMMANDS: &str = "commands";

/// The "cmd" debug level for "garden cmd".
pub const DEBUG_LEVEL_CMD: &str = "cmd";

/// The "config" debug level for configuration reading.
pub const DEBUG_LEVEL_CONFIG: &str = "config";

/// The "exec" debug level for "garden exec".
pub const DEBUG_LEVEL_EXEC: &str = "exec";

/// The "list" debug level for "garden ls".
pub const DEBUG_LEVEL_LIST: &str = "list";

/// The "default-remote" key in a tree block defines the default remote.
pub const DEFAULT_REMOTE: &str = "default-remote";

/// The "depth" key in a tree block defines the git clone depth.
pub const DEPTH: &str = "depth";

/// The "description" key in a tree block describes the tree.
pub const DESCRIPTION: &str = "description";

/// The default "." tree query selects the tree in the current directory.
pub(crate) const DOT: &str = ".";

/// The "HOME" environment variable is used to resolve "~".
pub(crate) const ENV_HOME: &str = "HOME";

/// The "PATH" environment variable defines the command search path.
pub(crate) const ENV_PATH: &str = "PATH";

// The "PWD" environment variable conveys the current working directory.
pub(crate) const ENV_PWD: &str = "PWD";

/// The "environment" section of a garden, template or tree block defines
/// environment variables that are set in command environments.
pub const ENVIRONMENT: &str = "environment";

/// The "extend" key in a tree or template block is used to extends an existing
/// tree or template block.
pub const EXTEND: &str = "extend";

/// The "garden" section of the configuration defines global settings.
pub const GARDEN: &str = "garden";

/// The "gardens" section of the configuration defines named "gardens" that
/// are used to aggregate trees, define expression variables and export
/// environment variables for commands.
pub const GARDENS: &str = "gardens";

/// The default "garden.yaml" configuration file.
pub const GARDEN_CONFIG: &str = "garden.yaml";

/// Builtin variable for the "garden.yaml" configuration directory.
pub const GARDEN_CONFIG_DIR: &str = "GARDEN_CONFIG_DIR";

/// Variable expression for the "garden.yaml" configuration directory.
pub const GARDEN_CONFIG_DIR_EXPR: &str = "${GARDEN_CONFIG_DIR}";

/// Builtin variable for the "garden.root" location where trees are grown.
pub const GARDEN_ROOT: &str = "GARDEN_ROOT";

/// Command-line defines for overriding configurable behavior.
pub(crate) const GARDEN_INTERACTIVE_SHELL: &str = "garden.interactive-shell";
pub(crate) const GARDEN_SHELL: &str = "garden.shell";
pub(crate) const GARDEN_SHELL_ERREXIT: &str = "garden.shell-errexit";
pub(crate) const GARDEN_SHELL_WORDSPLIT: &str = "garden.shell-wordsplit";
pub(crate) const GARDEN_TREE_BRANCHES: &str = "garden.tree-branches";

/// The "gitconfig" section in a tree block defines local ".git/config"
/// settings that are applied when a tree is grown.
pub const GITCONFIG: &str = "gitconfig";

/// The "grafts" section of the "garden" block is used to graft entities
/// from other garden files into the configuration under a custom namespace.
pub const GRAFTS: &str = "grafts";

/// The "groups" section of the configuration defines named groups of trees.
pub const GROUPS: &str = "groups";

/// The "includes" key in the garden block reads additional configuration
/// files directly into the configuration.
pub const INCLUDES: &str = "includes";

/// The "interactive-shell" key in the garden block overrides the
/// command used by interactive "garden shell" sessions.
pub const INTERACTIVE_SHELL: &str = "interactive-shell";

/// The "links" key in a tree block defines URLs displayed by "garden ls".
pub const LINKS: &str = "links";

/// The "origin" remote is the default Git remote name.
pub(crate) const ORIGIN: &str = "origin";

/// The "path" key in a tree block defines the location for a tree.
/// A directory relative to "garden.root" named after the tree is used as
/// the tree's path by default.
pub const PATH: &str = "path";

/// The "remotes" key in a tree block defines the Git remotes to configure when
/// a tree is grown.
pub const REMOTES: &str = "remotes";

/// The "replace" key in a tree block is used to completely replace a tree when
/// the tree was already loaded via an "includes" entry. The default behavior
/// is to merge and override settings when the same-named tree entries is
/// encountered.
pub const REPLACE: &str = "replace";

/// The "root" key in the garden block defines where trees are located and grown.
pub const ROOT: &str = "root";

/// The "shell" key in the garden block defines the shell to use for commands.
pub const SHELL: &str = "shell";

/// Fast all-in-one JavaScript runtime
pub(crate) const SHELL_BUN: &str = "bun";

/// GNU Bourne-Again Shell.
pub(crate) const SHELL_BASH: &str = "bash";

/// Dash command interpreter is the default shell on Debian.
pub(crate) const SHELL_DASH: &str = "dash";

/// The "shell-errexit" key in the garden block disables the "exit on error"
/// shell option.
pub const SHELL_ERREXIT: &str = "shell-errexit";

/// The "shell-wordsplit" key in the garden block disables the `zsh -o shwordsplit` option.
pub const SHELL_WORDSPLIT: &str = "shell-wordsplit";

/// KornShell is a standard/restricted command and programming language.
pub(crate) const SHELL_KSH: &str = "ksh";

/// Cross-platform JavaScript runtime environment.
pub(crate) const SHELL_NODE: &str = "node";

/// Cross-platform JavaScript runtime environment.
pub(crate) const SHELL_NODEJS: &str = "nodejs";

/// Highly capable, feature-rich programming language with over 36 years of development.
pub(crate) const SHELL_PERL: &str = "perl";

/// Default command interpreter.
pub(crate) const SHELL_SH: &str = "sh";

/// A dynamic, open source programming language with a focus on simplicity and productivity.
pub(crate) const SHELL_RUBY: &str = "ruby";

/// Extended version of the Bourne Shell with new features.
pub(crate) const SHELL_ZSH: &str = "zsh";

/// The "single-branch" key in a tree block is used to make "garden grow"
/// track only a single branch. Tracking branches for all remote branches
/// are cloned and fetched by default.
pub const SINGLE_BRANCH: &str = "single-branch";

/// The "symlink" key in a tree block creates a symlink.
pub const SYMLINK: &str = "symlink";

/// The "templates" section defines tree templates that can be used when
/// defining tree entries.
pub const TEMPLATES: &str = "templates";

/// The "tree-branches" key in the garden block can disable the current
/// branch indicator when trees are displayed.
pub const TREE_BRANCHES: &str = "tree-branches";

/// Builtin variable for tree names.
pub const TREE_NAME: &str = "TREE_NAME";

/// Builtin variable for tree paths.
pub const TREE_PATH: &str = "TREE_PATH";

/// The "trees" section of the configuration defines trees that can be cloned
/// into existence using "garden grow" and operated upon with custom commands.
pub const TREES: &str = "trees";

/// The "url" key in a tree block defines the "git clone" URL to clone.
pub const URL: &str = "url";

/// The "variables" section in a configuration block defines expression
/// variables that can be references using "${variable}" expressions in
/// "environment", "commands" and "variables" blocks. Variables
/// can use "$ exec" expressions to capture stdout from a command.
pub const VARIABLES: &str = "variables";

/// The "worktree" key in a tree block is used to refer to a parent
/// tree that will be used to grow the tree using "git worktree add".
pub const WORKTREE: &str = "worktree";
