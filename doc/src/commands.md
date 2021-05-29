# Commands

Garden has a handful of built-in commands and provides a flexible way to
extend it with user-defined commands. User-defined commands are one of the
most useful Garden features.


## Command-Line Conventions

All builtin garden commands have this basic syntax:

    garden [options] <command> [command-options] [command-arguments]*

The following options are specified before `<command>`, and are
global to all `garden` commands.

    -c | --config <path>

Specify a garden config file to use instead of searching for `garden.yaml`.
The path can either be the path to an actual config file, or it can be
the basename of a file in the configuration search path.

    -v | --verbose

Enable verbose debugging output.


    -s | --set  name=value

Override a configured variable by passing a `name=value` string to
the `--set` option.  The variable named `name` will be updated with the
garden expression `value`.  Multiple variables can be set by specifying the
flag multiple times.


## garden add

    garden add <path>

    # example
    garden add src/repo

Add an existing Git tree at `<path>` to `garden.yaml`.


## garden cmd

    garden cmd <query> <command>... [-- <arguments>..]

    # example
    garden cmd cola build test -- V=1

Run commands over the trees matched by `<query>`.

`garden cmd` and `garden <command>` interact with custom commands that are
configured in the "commands" section for templates, trees, gardens,
and the global top-level scope.

Custom commands can be defined at either the tree or garden level.
Commands defined at the garden level extend commands defined on a tree.

Commands are executed in a shell, so shell expressions can be used in commands.
Each command runs under `["sh", "-c", "<command>"]` with the resolved
environment from the corresponding garden, group, or tree.

Optional command-line `<arguments>` specified after a double-dash (`--`)
end-of-options marker are forwarded to each command.

`"$0"` in a custom command points to the current garden executable and can be
used to rerun garden from within a garden command.

Optional arguments are available to command strings by using the traditional
`"$@"` shell expression syntax.  When optional arguments are present `"$1"`,
`"$2"`, and subsequent variables will be set according to each argument.


### Custom Commands

    garden <command> <query>... [-- <arguments>...]

    # example
    garden status @git-cola .
    garden build cola -- V=1

When no builtin command exists by the specified name then garden will
use custom commands defined in a "commands" block at the corresponding
garden or tree scope.

This is complementary to `garden cmd <query> <command>...`.
That form can run multiple commands; this form operates over multiple queries.


## garden exec

    garden exec <query> <command> [<arguments>]*

    # example
    garden exec cola git status -s

Execute arbitrary commands on all of the trees matched by `<query>`.


## garden eval

    garden eval <expression> [<tree>] [<garden>]

    # example
    garden eval '${GARDEN_ROOT}'
    garden eval '${TREE_PATH}' cola

Evaluate a garden expression in the specified tree context.
If no tree is given then the variable scope includes the top-level variables
block only.

When a tree is given then its variables are considered as well.
When a garden is specified then the garden's variables are also available for
evaluation.


## garden init

    garden init [options]

    # create a local garden config rooted at the current directory
    garden init --root '${GARDEN_CONFIG_DIR}'

    # create a global garden config rooted at ~/src
    garden init --global --root '~/src'

Create a new empty `garden.yaml` in the current directory, or in the
user's global configuration directory when `--global` is specified.
See `garden help init` for more details.


## garden grow

    garden grow <query>

    # example
    garden grow cola

Update or create the tree(s) referenced by the `<query>`.

It is safe to re-run the `grow` command.  Existing trees will have their git
configuration updated to match the configured remotes.  Missing repositories
are created by cloning the configured tree url.


## garden shell

    garden shell <query> [<tree>]

    # example
    garden shell cola

Launch a shell inside the environment formed by the tree query.
The optional tree argument specifies which tree to chdir into.

If the resolved tree query contains a tree whose name exactly matches the
query string then that tree's directory will be used when opening the shell.
The optional tree argument is not needed for the common case where a garden
and tree share a name -- garden will chdir into that same-named tree when
creating the shell.
