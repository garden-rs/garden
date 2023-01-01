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

    -D | --define name=value

Override a configured variable by passing a `name=value` string to
the `-D | --define` option.  The variable named `name` will be updated with the
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

`garden plant` records the Git remotes associated with a repository.

Repositories created using `git worktree` are supported by `garden plant`.
Parent trees must be planted first before planting a child tree.


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

### Branches

The `branch: <branch-name>` tree variable is used to specify which branch should be
cloned and checked out when the tree is grown.

    trees:
      example:
        branch: dev
        url: <url>

`graden grow example` clones the repository using `git clone --branch=dev`.
The `branch` setting is a tree variable and supports `${variable}` expressions.


### Shallow Clones

The `depth: <integer>` tree parameter is used to create shallow clones.

    trees:
      example:
        depth: 42
        url: <url>

`garden grow example` clones the repository using:

    git clone --depth=42 --no-single-branch

Even though a shallow clone is created, all of the remote tracking branches
(eg. `origin/*`) are available because we clone the repository using
the `--no-single-branch` option.

The `single-branch: true` tree parameter is used to create clones that contain
a single branch only. This is useful if you want to limit the on-disk footprint
of repositories by only having a single branch available.

This paramter is typically used in conjunction with `branch: <branch-name>` and
`depth: 1` to create a 1-commit shallow clone with a single branch.

    trees:
      example:
        branch: dev
        depth: 1
        single-branch: true
        url: <url>


### Wildcards

Wildcards are supported in the trees queries supported by `garden grow`.
`garden grow 'glob*'` grows the gardens, groups or trees that start with "glob".

If `garden.yaml` contains `gardens` whose name matches the query then the trees
associated with each garden are grown.

If no gardens are found then garden will search for "groups" that match
the query. If groups are found then the trees within each group will be grown.

If no gardens and no groups are found then will garden search for trees and grow
those whose names match the query string.


### Worktrees

`garden grow` can be used to create worktrees that share their `.git` storage
using [git worktree](https://git-scm.com/docs/git-worktree).

To create shared storage, define the primary worktree where the `.git`
storage will reside and then define additional trees that reference the
worktrees.

    trees:
      example/main: <url>

      example/dev:
        worktree: example/main
        branch: dev

    example/v2:
        worktree: example/main
        branch: v2

This example uses `example/main` tree for the shared storage and two additional worktrees.
`example/dev` uses the `dev` branch and `example/v2` uses the `v2` branch.


### Bare Repositories

To clone bare repositories use `bare: true` in the tree configuration.

    trees:
      example:
        bare: true
        url: <url>

Bare clones are created by default when the tree path ends in `.git`.
For example, a tree called `example.git` will be `bare: true` by default.

    trees:
      example.git: <url>

Setting `bare: false` overrides the name-based detection of bare repositories.

    trees:
      example.git:
        bare: false
        url: <url>


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

### Commands

`garden cmd` and `garden <command>` interact with custom commands that are
configured in the `commands` section for templates, trees, gardens,
and the top-level scope.

Custom commands can be defined at either the tree or garden level.
Commands defined at the garden level extend commands defined on a tree.
If both a tree and the garden containing that tree defines a command called
`test` then `garden test` will first run the tree's `test` command followed
by the garden's `test` command.

Commands are executed in a shell so that shell expressions can be used in commands.
A POSIX-compatible shell must be installed in your `$PATH`.

The `garden.shell` configuration value defaults to `zsh` but can be set to any
shell that accepts `-e` and `-c '<command>` options (for example `bash`).
If `zsh` is not installed then `bash` will be used by default instead.
If neither `zsh` nor `bash` is installed then `sh` will be used by default instead.

Each command runs under `["zsh", "-e", "-c", "<command>"]` with the resolved
environment from the corresponding garden, group, or tree.

Multi-line and multi-statement command strings will stop executing as soon as the
first non-zero exit code is encountered due to the use of the `-e` shell option.
Use the `-n | --no-errexit` option to inhibit the use of the `-e` errexit option.

The `--no-errexit` option causes commands with multiple statements to run to completion
even when a non-zero exit code is encountered. This is akin to a regular shell script.

You can also opt-out of the `errexit` behavior on a per-command basis by adding
`set +e` as the first line of a multi-line command.

Additional command-line `<arguments>` specified after a double-dash (`--`)
end-of-options marker are forwarded to each command.

`"$0"` in a custom command points to the current garden executable and can be
used to rerun garden from within a garden command.

Additional arguments are available to command strings by using the traditional
`"$@"` shell syntax.  When additional arguments are present `"$1"`, `"$2"`, and
subsequent variables will be set according to each argument.

    # Example usage
    garden test cola -- V=1

### Depth-first and Breadth-first Tree Traversal

The following two invocations run commands in a different order:

    # Depth-first (default)
    garden cmd treesitters build test

    # Breadth-first
    garden cmd --breadth-first treesitters build test

The default traversal order for commands is depth-first. This means that *both* the
`build` and `test` commands are run on each tree in the `treesitters` group
*before* running any commands on the next tree.

The `-b | --breadth-first` option enables a breadth-first traversal. A breadth-first
traversal runs the `build` command over *all* of the trees in the `treesitters` group
*before* the `test` command is run over all of the trees in the same group.

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


## garden prune

    garden prune [options] [<subdirs>...]

Traverse the filesystem and interactively delete any repositories that are
not referenced by the garden file.

This command is intended to cleanup a garden-managed directory. Its intended
usage is to delete repositories that were created (eg. via `garden grow`) and
have since been removed from your version-controlled garden configuration.

**Warning**: `garden prune` is a dangerous command and must be run with care.
`garden prune` deletes repositories and all of their files (including the `.git` storage)!

The following options are supported by `garden prune`.

## Enable deletions

    --rm

The `garden prune` command uses a no-op "safe mode" which does not actually
delete repositories by default. Deletions must be enabled by specifying the
`--rm` option.

Use the `--rm` option only after you have verified that `garden prune` is not
going to delete any unexpected repositories that you intended to keep.

## Limit concurrency

    --jobs <jobs>

The prune process runs in parallel across multiple cores. All cores are used by default.
The `--jobs` option limits the number of cores to the specified number of jobs.

## Limit filesystem traversal depth

    --min-depth <minimum-depth>
    --max-depth <maximum-depth>
    --exact-depth <exact-depth>

The cleanup process can be limited to specific traversal depths. The filesystem is
traversed with no limits by default.

Specifying a minimum depth will not remove repositories shallower than the specified
depth. For example, `--min-depth 1` will not remove repositories in the same directory
as the garden file.

Specifying a maximum depth will not remove repositories deeper than the specified
depth. For example, `--max-depth 0` will not remove repositories in subdirectories
below the directory containing the graden file.

## Enable scripted usage by answering "yes" to all prompts

    --no-prompt

The `garden prune` command interactively prompts before removing each repository.
The prompt looks like the following:

    # /home/user/src/example
    Delete the "example" repository?
    WARNING: "all" deletes "example" and ALL subsequent repositories!
    Choices: yes, no, all, quit [y,n,all,q]?

Entering `y` (or `yes`) at the prompt will delete the repository and all of its files.

Entering `n` (or `no`) at the prompt will skip and not remove the repository.

Entering `q` (or `quit`) will exit `garden prune` without deleting the repository.

Entering `all` will remove the repository and all subsequent repositories.
`all` is equivalent to answering `yes` to all further prompts.

Entering `all` is dangerous and proceeds without further prompts. Be careful!

`--no-prompt` is also equivalent to answering `yes` to all prompts.
`--no-prompt` is intended for use in scripts where user interaction is not desired.
Use with caution!
