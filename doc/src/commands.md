# Commands

Garden includes a set of built-in commands and can be flexibly extended
with user-defined commands. User-defined commands are one Garden's most
useful features.


## Command-Line Conventions

Run `garden help` to display usage information for garden commands.
The usage information is where the command-line options are documented.

```bash
garden help
garden help <command>
garden <command> --help
```

Built-in commands use this basic syntax:

    garden [options] <command> [command-options] [command-arguments]*

The following options come before `<command>` and are common to all commands.

    -C | --chdir <directory>

Navigate to the specified directory before searching for configuration.
This is modeled after `make -C <path> ...` or `git -C <path> ...`.

    -c | --config <filename>

Specify a garden file to use instead of searching for `garden.yaml`.
The filename can be either the path to a file or the basename of a file in the
configuration search path.

    -v | --verbose

Enable verbose debugging output.

    -D | --define name=value

Override a configured variable by passing a `name=value` string to
the `-D | --define` option.  The variable named `name` will be updated with the
garden expression `value`.  Multiple variables can be set by specifying the
flag multiple times.


## garden init

```bash
garden init [options] [<filename>]

# create a local garden config rooted at the current directory
garden init --root '${GARDEN_CONFIG_DIR}'

# create a global garden config rooted at ~/src
garden init --global --root '~/src'
```

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

```bash
garden plant <tree>
```

Add a pre-existing Git worktree to `garden.yaml`.

The `trees` section in the `garden.yaml` file will be updated with details
about the new tree.

`garden plant` records the Git remotes associated with a repository.
It is safe to re-run `garden plant` in order to add new remotes to
an existing configuration.

Repositories created using `git worktree` are supported by `garden plant`.
Parent trees must be planted first before planting a child tree.

Use the `--sort` option to sort all of the `trees` entries after planting.


## garden ... [tree-query]

Garden commands accept [tree query](tree-queries.md) strings that are used to
refer to sets of trees.

Tree queries are glob string patterns that can be used to match the gardens,
groups or trees defined in "garden.yaml".


## garden grow

```bash
garden grow <tree-query>

# Example usage
garden grow cola
```

If you have a `garden.yaml` file, either one that you authored yourself or
one that was provided to you, then you will need to grow the Git trees
into existence.

The `grow` sub-command updates or creates the trees matched by the `<tree-query>`
and places them into the paths defined by the garden file.

It is safe to re-run the `grow` command and re-grow a tree.  Existing trees will
have their git configuration updated to match the configured remotes.  Missing
repositories are created by cloning the configured tree URL.

### Branches

The `branch: <branch-name>` tree variable is used to specify which branch should be
cloned and checked out when the tree is grown.

```yaml
trees:
  example:
    branch: dev
    url: url
```

`garden grow example` clones the repository using `git clone --branch=dev`.
The `branch` setting is a tree variable and supports `${variable}` expressions.


### Shallow Clones

The `depth: <integer>` tree parameter is used to create shallow clones.

```yaml
trees:
  example:
    depth: 42
    url: url
```

`garden grow example` clones the repository using:

```bash
git clone --depth=42 --no-single-branch
```

Even though a shallow clone is created, all of the remote tracking branches
(e.g. `origin/*`) are available because we clone the repository using
the `--no-single-branch` option.

The `single-branch: true` tree parameter is used to create clones that contain
a single branch only. This is useful if you want to limit the on-disk footprint
of repositories by only having a single branch available.

This par mater is typically used in conjunction with `branch: <branch-name>` and
`depth: 1` to create a 1-commit shallow clone with a single branch.

```yaml
trees:
  example:
    branch: dev
    depth: 1
    single-branch: true
    url: url
```


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

```yaml
trees:
  example/main: url

  example/dev:
    worktree: example/main
    branch: dev

  example/v2:
    worktree: example/main
    branch: v2
```

This example uses `example/main` tree for the shared storage and two additional worktrees.
`example/dev` uses the `dev` branch and `example/v2` uses the `v2` branch.


### Upstream Branches

Remote tracking branches can be configured by defining a `branches` block that maps
local branch names (the `key`) to a remote branch name expression (the `value`).

```yaml
trees:
  example/branches:
    branch: local
    branches:
      local: origin/dev
      dev: origin/dev
```

The above configuration creates local branches called `local` and `dev` and checks out
the `local` branch when `garden grow example/branches` is run.

### Git Configuration Values

The `garden grow` command will apply git configuration values that are
specified using the `gitconfig` key.

```yaml
trees:
  foo:
    gitconfig:
      # Override the submodule URL for "thirdparty/repo".
      submodule.thirdparty/repo.url: "git@private.example.com:thirdparty/repo.git"
```

Multi-valued `git config --add` values are also supported, for example
the [remote.$name.pushurl](https://git-scm.com/docs/git-config#Documentation/git-config.txt-remoteltnamegtpushurl)
value can be set to multiple values.

Use a list as the value for the key and multiple values will be added using
[git config --add $name $value](https://git-scm.com/docs/git-config#Documentation/git-config.txt---add).

```yaml
trees:
  foo:
    url: "git@github.com:example/foo.git"
    gitconfig:
      remote.origin.pushurl:
        # Push to multiple remotes when using "git push origin"
        - "git@github.com:example/foo.git"
        - "git@private.example.com:example/foo.git"
```

### Bare Repositories

To clone bare repositories use `bare: true` in the tree configuration.

```yaml
trees:
  example:
    bare: true
    url: url
```

Bare clones are created by default when the tree path ends in `.git`.
For example, a tree called `example.git` will be `bare: true` by default.

```yaml
trees:
  example.git: url
```

Setting `bare: false` overrides the name-based detection of bare repositories.

```yaml
trees:
  example.git:
    bare: false
    url: url
```

## garden cmd

```bash
garden cmd <tree-query> <command> [<command>]... [-- <arguments>..]

# Example usage
garden cmd cola build test -- V=1
```

Run one or more user-defined commands over the gardens, groups or trees
that match the specified tree query.

The example above runs the `build` and `test` commands in all of the trees
that are part of the `cola` garden.

### Commands

`garden cmd` and `garden <command>` interact with custom commands that are
configured in the `commands` section for templates, trees, gardens,
and the top-level scope.

```bash
# Example usage

# Run the test" command in the cola and vx trees
garden test cola vx

# Use "--" to forward arguments to the custom commands
garden test cola vx -- V=1
```

Custom commands can be defined at either the tree or garden scope.
Commands defined at the garden scope extend commands defined on a tree.
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
Use the `-n | --no-errexit` option to inhibit the use of the `-e` "errexit" option.

The `--no-errexit` option causes commands with multiple statements to run to completion
even when a non-zero exit code is encountered. This is akin to a regular shell script.

Configure `garden.shell-errexit` to `false` in `garden.yaml` to opt-out of this behavior.
You can also opt-out of the `errexit` behavior on a per-command basis by adding
`set +e` as the first line of a multi-line command.

Additional command-line `<arguments>` specified after a double-dash (`--`)
end-of-options marker are forwarded to each command.

`"$0"` in a custom command points to the current garden executable and can be
used to rerun garden from within a garden command.

Additional arguments are available to command strings by using the traditional
`"$@"` shell syntax.  When additional arguments are present `"$1"`, `"$2"`, and
subsequent variables will be set according to each argument.

```yaml
# Commands can be defined in multiple ways.
# Strings and lists of strings are both supported via "String to List Promotion".
# The YAML reader accepts multi-line strings using the "|" pipe syntax.

commands:
  one-liner: echo hello "$@"
  multi-liner: |
    if test "${debian}" = "yes"
    then
        apt install rust-all
    else
  multi-statement-and-multi-liner:
    - echo a $1
    - |
      echo b $3
      echo c $4

variables:
  name: value
  debian: $ type apt-get >/dev/null && echo yes || echo no

# Commands can also be defined at tree and garden scope

trees:
  our-tree:
    commands:
      tree-cmd: echo ${TREE_NAME}

gardens:
  all:
    trees: *
    commands:
      print-pwd: pwd
```

### Shell Syntax

User-defined Commands and Exec Expressions are evaluated by the shell configured
in the `garden.shell` configuration value.

Garden and Shells share a key piece of common syntax: the interpolated braced
`${variable}` syntax.

Garden Variables that use the `${variable}` syntax are evaluated by
`garden` first before the shell has evaluated them.

This means that the shell syntax supported by Garden's Exec Expressions is
a subset of the full syntax because shell-only variables such as `${OSTYPE}` cannot
be accessed using the braced-variable syntax.

```yaml
commands:
  example-command: |
    shell_variable=$(date +%s)
    echo hello $OSTYPE $shell_variable
```

The plain `$variable` syntax is reserved for use by the shell commands used in
user-defined Commands and Exec Expressions.

Environment Variables can be used in shell scriptlets through both the `$ENV` and
`${ENV}` braced variable syntax. Garden makes all environment variables available during
variable expansion.

The distinction between the `${garden}` and `$shell` syntax is only relevant when
using variables defined within shell command, such as `$shell_variable` above.

If the `${shell_variable}` syntax were to be used in the `example-command` then an
empty value would have been used instead of the output of `date +%s`.

Sometimes it is necessary to actually use the `${...}` braced literal syntax
in shell commands. The `$${...}` braced double-dollar syntax can be used to
escape a braced value and disable evaluation by `garden`.

Double-`$` can generally be used to escape literal `$` values in commands, but
escaping is handled automatically for regular `$shell` variables.

### Depth-first and Breadth-first Tree Traversal

The following two invocations run commands in a different order:

```bash
# Depth-first (default)
garden cmd treesitters build test

# Breadth-first
garden cmd --breadth-first treesitters build test
```

The default traversal order for commands is depth-first. This means that *both* the
`build` and `test` commands are run on each tree in the `treesitters` group
*before* running any commands on the next tree.

The `-b | --breadth-first` option enables a breadth-first traversal. A breadth-first
traversal runs the `build` command over *all* of the trees in the `treesitters` group
*before* the `test` command is run over all of the trees in the same group.

### Custom Commands

```bash
garden <command> <query> [<query>]* [-- <arguments>...]

# Example usage
garden status @git-cola .
garden build cola -- V=1
```

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

Use the `garden -vv` extra-verbose option to display the commands being run.

### Pre and Post Commands

Commands can specify references to other commands that should be run before and/or after
a command.

* Pre-commands are run before the command.

* Pre-commands use a `<` suffix with values that specify the names of other commands to
  run before the command.

* Post-commands are run after the command.

* Post-commands use a `>` suffix with values that specify the names of other commands to
  run after the command.

* Pre-commands and post-commands can only refer to other custom commands.

```yaml
commands:
  custom-cmd: echo custom-cmd
  custom-cmd<: pre
  custom-cmd>:
    - post1
    - post2
  pre: echo before
  post1: echo after1
  post1: echo after2
```

Running `garden custom-cmd` with the above configuration runs the following commands:

```
# pre
echo before

# custom-cmd
echo custom-cmd

# post1
echo after1

# post2
echo after2
```


## garden exec

```bash
garden exec <tree-query> <command> [<arguments>]*

# example
garden exec cola git status -s
```

Run commands over the trees, groups or gardens matched by tree query.
When the `<tree-query>` resolves to a garden then the environment
is configured for the command using the environment variables and
custom commands from both the tree and the garden.

Use the `garden -vv` extra-verbose option to display the command being run.

Use the `--dry-run` / `-n` option to perform a trial run without running any commands.


## garden eval

```bash
garden eval <expression> [<tree>] [<garden>]

# example
garden eval '${GARDEN_ROOT}'
garden eval '${TREE_PATH}' cola
```

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


## garden ls

    garden ls [options] [<tree-query>...]

Display configured information about trees and commands.
Tree details are not displayed for missing / ungrown trees.
Use the `-a` option to display details for missing trees.

If no tree-queries are specified then `garden ls` behaves as if
`garden ls '@*'` were specified, which displays all trees.


## garden prune

    garden prune [options] [<subdirs>...]

Traverse the filesystem and interactively delete any repositories that are
not referenced by the garden file.

This command is intended to cleanup a garden-managed directory. Its intended
usage is to delete repositories that were created (e.g. via `garden grow`) and
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
below the directory containing the garden file.

## Enable scripted usage by answering "yes" to all prompts

    --no-prompt

The `garden prune` command interactively prompts before removing each repository.
The prompt looks like the following:

```bash
# /home/user/src/example
Delete the "example" repository?
WARNING: "all" deletes "example" and ALL subsequent repositories!
Choices: yes, no, all, quit [y,n,all,q]?
```

Entering `y` (or `yes`) at the prompt will delete the repository and all of its files.

Entering `n` (or `no`) at the prompt will skip and not remove the repository.

Entering `q` (or `quit`) will exit `garden prune` without deleting the repository.

Entering `all` will remove the repository and all subsequent repositories.
`all` is equivalent to answering `yes` to all further prompts.

Entering `all` is dangerous and proceeds without further prompts. Be careful!

`--no-prompt` is also equivalent to answering `yes` to all prompts.
`--no-prompt` is intended for use in scripts where user interaction is not desired.
Use with caution!


## garden completion

Shell completions for `garden` can be generated by running the `garden completion`
command.

`garden completion` uses [clap complete](https://crates.io/crates/clap_complete)
to generate its completions.

The `--commands` options will additionally generate completions for custom commands.

### Zsh

Ensure that your `~/.zshrc` file has completions enabled and that you have a
directory configured in your `$fpath` for `zsh` completion scripts.
Add the following snippet to your `~/.zshrc` to enable completions and
configure `~/.config/zsh/completion` for providing completion scripts.

```bash
fpath=(~/.config/zsh/completion $fpath)
autoload -U +X compinit
compinit -i
```

These settings make `zsh` look for a script called `_garden` in the directory when
tab-completing for `garden`.

Lastly, create the directory and generate the `~/.config/zsh/completion/_garden`
zsh shell completion script.

```bash
mkdir -p ~/.config/zsh/completion
garden completion zsh >~/.config/zsh/completion/_garden
```

Use `garden completion --commands zsh` instead of `garden completion zsh`
to include completions for custom commands.

*NOTE*: You should regenerate the `_garden` zsh completion script whenever `garden`
is upgraded to ensure that all of the options and commands have up to date completions.

### Bash

Add the following snippet to your `~/.bashrc` to enable `bash` tab completions.

```bash
if test -n "$PS1" && type garden >/dev/null 2>&1
then
    eval "$(garden completion bash)" 2>/dev/null
fi
```

Use `garden completion --commands bash` instead of `garden completion bash`
to include completions for custom commands.

### Future shell completion enhancements

Tab completion can only be made to include a static set of user-defined commands.
Custom commands cannot be defined dynamically, which means that the same completions
will be used irrespective of your current directory.

Improvements to the shell completions can be made once traction has been made on the
following upstream issues:

* [clap #3022](https://github.com/clap-rs/clap/issues/3022) - zsh broken with two multi length arguments

* [clap #4612](https://github.com/clap-rs/clap/pull/4612) - candidate fix for the above issue

* [clapng #92](https://github.com/epage/clapng/issues/92) - Dynamic completion support
