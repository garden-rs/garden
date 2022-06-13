# Commands

Garden includes a set of built-in commands and can be flexibly extended
with user-defined commands. User-defined commands are one Garden's most
useful features.


## Command-Line Conventions

Run `garden help` to display usage information for garden commands.
The usage information is where the command-line options are documented.

    garden help
    garden help <command>
    garden <command> --help

Built-in commands use this basic syntax:

    garden [options] <command> [command-options] [command-arguments]*

The following options come before `<command>` and are common to all commands.

    -C | --chdir <directory>

chdir to the specified directory before searching for configuration.
This is modeled after `make -C <path> ...` or `git -C <path> ...`.

    -c | --config <filename>

Specify a garden config file to use instead of searching for `garden.yaml`.
The path can either be the path to an actual config file, or it can be
the basename of a file in the configuration search path.

    -v | --verbose

Enable verbose debugging output.

    -s | --set name=value

Override a configured variable by passing a `name=value` string to
the `--set` option.  The variable named `name` will be updated with the
garden expression `value`.  Multiple variables can be set by specifying the
flag multiple times.


## garden init

    garden init [options] [<filename>]

    # create a local garden config rooted at the current directory
    garden init --root '${GARDEN_CONFIG_DIR}'

    # create a global garden config rooted at ~/src
    garden init --global --root '~/src'

The init command will create an empty Garden YAML file with the minimal
boilerplate to start using garden. If no `<filename>` is specified,
`garden.yaml` in the current directory will be written.

The Garden file is written to the user's `~/.config/garden/` global configuration
directory when `--global` is specified.

This command is typically run without specifying a filename.
After `garden init` the following files are created.

```
current-directory/
└── garden.yaml
```


## garden plant

    garden plant <tree>

Add a pre-existing Git worktree to `garden.yaml`.

The `trees` section in the `garden.yaml` file will be updated with details
about the new tree.


## garden ... [tree-query]

Garden commands accept [tree query](tree-queries.md) strings that are used to
refer to sets of trees.

Tree queries are glob string patterns that can be used to match the gardens,
groups or trees defined in "garden.yaml".


## garden grow

    garden grow <tree-query>

    # Example usage
    garden grow cola

If you have a `garden.yaml` file, either one that you authored yourself or
one that was provided to you, then you will need to grow the Git trees
into existence.

The `grow` sub-command updates or creates the trees matched by the `<tree-query>`
and places them into the paths defined by the garden file.

It is safe to re-run the `grow` command and re-grow a tree.  Existing trees will
have their git configuration updated to match the configured remotes.  Missing
repositories are created by cloning the configured tree url.

The `depth: <integer>` tree parameter is used to create shallow clones.

    trees:
      example:
        depth: 42
        url: <url>

`garden grow example` clones the repository using `git clone --depth=42 <url>`.

`garden grow 'glob*'` grows the gardens, groups or trees that start with "glob".

If "garden.yaml" contains "gardens" whose name matches the query then the trees
associated with each garden are grown.

If no gardens are found then garden will search for "groups" that match
the query. If groups are found then the trees within each group will be grown.

If no gardens and no groups are found then will garden search for trees and grow
those whose names match the query string.


## garden cmd

    garden cmd <tree-query> <command> [<command>]... [-- <arguments>..]

    # Example usage
    garden cmd cola build test -- V=1

Run one or more user-defined commands over the gardens, groups or trees
that match the specified tree query.

For example if you have a group called `treesitters` and two custom commands
called `build` and `test`, then running `garden cmd treesitters build test`
will run the custom `build` and `test` commands over all of the trees in
`treesitters` group.

`garden cmd` and `garden <command>` interact with custom commands that are
configured in the `commands` section for templates, trees, gardens,
and the top-level scope.

Custom commands can be defined at either the tree or garden level.
Commands defined at the garden level extend commands defined on a tree.
If both a tree and the garden containing that tree defines a command called
"test" then "garden test" will first run the tree's "test" command followed
by the garden's "test" command.

Commands are executed in a shell and shell expressions can be used in commands.
Each command runs under `["sh", "-c", "<command>"]` with the resolved
environment from the corresponding garden, group, or tree.

Additional command-line `<arguments>` specified after a double-dash (`--`)
end-of-options marker are forwarded to each command.

`"$0"` in a custom command points to the current garden executable and can be
used to rerun garden from within a garden command.

Additional arguments are available to command strings by using the traditional
`"$@"` shell syntax.  When additional arguments are present `"$1"`, `"$2"`, and
subsequent variables will be set according to each argument.

    # Example usage
    garden test cola -- V=1


### Custom Commands

    garden <command> <query> [<query>]* [-- <arguments>...]

    # Example usage
    garden status @git-cola .
    garden build cola -- V=1

`garden <command>` is another way to execute a user-defined `<command>`.
This form allows you to specify multiple queries rather than multiple commands.

When no builtin command exists by the specified name then garden will
use custom commands defined in a `commands` block at the corresponding
garden or tree scope.

`garden <command> <query>...` is complementary to `garden cmd <query> <command>...`.

`garden cmd ...` runs multiple commands over a single query.

`garden <command> ...` runs a command over multiple queries.

For example, `garden build treesitters catsitters` will run a user-defined `build`
command over both the `treesitters`  and `catsitters` groups.


## garden exec

    garden exec <tree-query> <command> [<arguments>]*

    # example
    garden exec cola git status -s

Run commands over the trees, groups or gardens matched by tree query.
When the `<tree-query>` resolves to a garden then the environment
is configured for the command using the environment variables and
custom commands from both the tree and the garden.


## garden eval

    garden eval <expression> [<tree>] [<garden>]

    # example
    garden eval '${GARDEN_ROOT}'
    garden eval '${TREE_PATH}' cola

Evaluate a Garden Expression in the specified tree context and output
the result to stdout.

Garden Expressions are `strings-with-${variables}` that get substituted
and resolved using garden's `variables` and `environment` blocks.

If no tree is given then the variable scope includes the top-level variables
block only.

When a tree is given then its variables are considered as well.

When a garden is specified then the garden's variables are also available for
evaluation.


## garden shell

    garden shell <tree-query> [<tree>]

    # example
    garden shell cola

Launch a shell inside the environment synthesized by the tree query.
If `<tree>` is specified then the current directory will be set to the
tree's directory.

If the resolved tree query contains a tree whose name exactly matches the
query string then that tree's directory will be used when opening the shell.
The optional tree argument is not needed for the case where a garden
and tree share a name -- garden will chdir into that same-named tree when
creating the shell.
