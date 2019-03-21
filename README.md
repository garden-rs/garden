# garden -- grow and cultivate collections of Git trees

Garden makes it easy to run operations over arbitrary collections of
self-contained Git trees.


## Use cases

* Make it easy to define collections of independent Git repositories and
  group them into cohesive development gardens.

* Build/test/install interdependent projects in self-contained sandboxes.

* Bootstrap development environments from simple configuration.

* Make it easy to recreate complex Git-based development environments from
  scratch using a simple configuration file.

Garden helps weave together development environments directly from Git trees.
Garden aids in common development setup steps such as setting environment
variables, search paths, and creating arbitrary groupings of repositories for
development.

Garden is used by creating a simple configuration file and defining a
directory structure for git trees.  Existing trees can be added to the
configuration using `gdn add`.

When bootstrapping from a pre-defined `garden.yaml`, the `gdn init` command
is used to bring trees into existence.

# Configuration

Garden is configured through a garden configuration file, typically called
"garden.yaml".  You can specify a config file by specifying
`-c|--config <filename>` when running garden, or arrange to have
garden find it in the current directory or in specific locations
on the filesystem.

Garden searches for "garden.yaml" in the following locations by default.

    # Relative to the current directory
    ./garden.yaml
    ./garden/garden.yaml
    ./etc/garden/garden.yaml

    # Relative to $HOME
    ~/.config/garden/garden.yaml
    ~/etc/garden/garden.yaml

    # Global configuration
    /etc/garden/garden.yaml


The following example `garden.yaml` is referred to by the documentation
when showing examples.

    garden:
      root: ~/src

    variables:
      flavor: debug
      prefix: ${TREE_PATH}/target/${flavor}
      num_procs: $ nprocs 2>/dev/null || echo 4

    commands:
      add: git add -u
      diff: GIT_PAGER= git diff
      cola: git cola
      fetch: git pull --ff-only
      gitk: gitk --all &
      log: git log
      pull: git pull --ff-only
      push: git push
      status:
        - git branch
        - git status --short

    templates:
      annex:
        gitconfig:
          remote.origin.annex-ignore: true
        commands:
          sync:
            - git fetch && git rebase origin/master
            - git annex sync --content
      rust:
        environment:
          PATH: ${TREE_PATH}/target/${rust_flavor}
        commands:
          build: cargo build
          test: cargo test
          doc: cargo doc
          run: cargo run
      bin:
        environment:
          PATH: ${TREE_PATH}/bin
      python:
        environment:
          PYTHONPATH: ${TREE_PATH}
      makefile:
        commands:
          build: make -j${num_procs} prefix="${prefix}" all
          install: make -j${num_procs} prefix="${prefix}" install
          test: make -j${num_procs} prefix="${prefix}" test
          doc: make -j${num_procs} prefix="${prefix}" doc

    trees:
      cola:
        path: git-cola
        url: git://github.com/git-cola/git-cola.git
        environment: {GIT_COLA_TRACE=: 1}
        templates: [bin, makefile, python]
        remotes:
          gitlab:
            url: git@gitlab.com:git-cola/git-cola.git

      garden:
        url: git://github.com/davvid/garden.git
        templates: rust

      garden/release:
        extend: garden
        path: garden
        variables:
          flavor: release

      git:
        url: git://github.com/git/git.git
        templates: makefile
        environment:
          PATH: ${TREE_PATH}/bin-wrappers

      gitk:
        url: git://ozlabs.org/~paulus/gitk.git
        templates: makefile
        environment:
          PATH: ${TREE_PATH}

      qtpy:
        url: git://github.com/spyder-ide/qtpy.git
        templates: python

      vx:
        url: git://github.com/davvid/vx.git
        environment:
          PATH: ${TREE_PATH}

      annex/music:
        url: git://git.example.com/music.git
        remotes:
          backup: user@backup:music
        templates: annex

      annex/movies:
        url: git://git.example.com/movies.git
        remotes:
          backup: user@backup:movies
        templates: annex

    groups:
      annex: annex/*
      cola: [cola, git, qtpy, vx]
      git-all:
       - cola
       - git*

    gardens:
      annex:
        trees: annex/*
      cola27:
        groups: cola
        commands:
          setup27:
            - virtualenv --system-site-packages env27
            - vx env27 make requirements
            - vx env27 make requirements-dev
          build27: vx env27 make all
          doc27: vx env27 make doc
          test27: vx env27 make test
          run27: vx env27 git cola
      gitdev:
        variables:
          prefix: ~/apps/gitdev
        environment:
          PATH: ~/apps/gitdev/bin
        trees: git*


### Garden Root

The garden root directory is configured in the `garden.root` field.
This directory is the parent directory beneath which all trees will be cloned.
Slashes in tree paths will create new directories on disk as needed.
`garden.root` defaults to the current directory when unspecified.


The built-in `${GARDEN_CONFIG_DIR}` variable can be used to create relocatable
setups that define a `garden.root` relative to the config file itself.

To place all trees in a `src` directory sibling to the `garden.yaml` file, the
following configuration can be used:

    garden:
      root: ${GARDEN_CONFIG_DIR}/src

# Variables

Garden configuration contains a "variables" block that allows defining
variables that are can be referenced by other garden values.

    variables:
      flavor: debug
      home: ~
      user: $ whoami
      libdir: $ test -e /usr/lib64 && echo lib64 || echo lib
      nproc: $ nproc
      prefix: ${TREE_PATH}/target/${flavor}
      py_ver_code: from sys import version_info as v; print("%s.%s" % v[:2])
      py_ver: $ python -c '${py_ver_code}'
      py_site: ${libdir}/python${py_ver}/site-packages

Variables definitions can reference environment variables and other garden
variables.

Variable references use shell `${variable}` syntax.

Values that start with dollar-sign+space (`$ `) are called "exec expressions".
Exec expressions are run through a shell after evaluation and replaced with
the output of the evaluated command.

When resolving values, variables defined in a tree scope override/replace
variables defined at the global scope.  Variables defined in garden scope
override/replace variables defined in a tree scope.

# Built-in variables

Garden automatically defines some built-in variables that can be useful
when constructing values for variables, commands, and paths.

    GARDEN_CONFIG_DIR   -   directory containing the "garden.yaml" config file
    GARDEN_ROOT         -   root directory for trees
    TREE_NAME           -   current tree name
    TREE_PATH           -   current tree path

## Environment Variables

The "environment" block defines variables that are stored in the environment.
Names with an equals sign (`=`) suffix are treated as literal values and
stored in the environment as-is.  Otherwise, the variable names are prepended
to using colons (`:`).  Use a plus sign (`+`) suffix in the name to append
to a variable instead of prepending.

Environment variables are resolved in the same order as the garden variables:
global scope, tree scope, and garden scope.  This allows gardens to
prepend/append variables after a tree, thus allowing for customization
of behavior from the garden scope.

Environment variables are resolved after garden variables.  This allows
the use of garden variables when defining environment variable values.

Environment variable names can use garden `${variable}` syntax when defining
their name, for example,

    trees:
      foo:
        environment:
          ${TREE_NAME}_LOCATION=: ${TREE_PATH}

exports a variable called `foo_LOCATION` with the location of the `foo` tree.


### OS Environment Variables

OS-level environment variables that are present in garden's runtime
environment can be referenced using garden `${variable}` expression syntax.
Garden variables have higher precedence than environment variables when
resolving `${variable}` references -- the environment is checked only when
no garden variables exist by that name.

### Gardens and Groups

Gardens aggregate groups and trees.  Define a group and reuse the group in
different gardens to share tree lists between gardens.  Defining gardens
and groups make those names available when querying for trees.

### Templates

Templates allow sharing of variables, gitconfig, and environments between
trees.  Trees can also reuse another tree definition by specifying the
"extend" keyword with the name of another tree.  Only the first remote is used
when extending a tree.

### Automagic Lists

Fields that expect lists can also be specified using a single string, and the
list will be treated like a list with a single item.  This is useful, for
example, when defining groups using wildcards, or commands which can sometimes
be one-lines, and multi-line at other times.

### Wildcards

The names in garden `tree` and `group` lists, and group member names accept glob
wildcard patterns.

The "annex" group definition is: `annex/*`.   This matches all trees that
start with "annex/".  The "git-all" group has two entries -- `git*` and
`cola`.  the first matches all trees that start with "git", and the second one
matches "cola" only.


### Symlinks

Symlink trees create a symlink on the filesystem during `garden init`.
`garden exec`, and custom `garden cmd` commands ignore symlink trees.

    trees:
      media:
        path: ~/media
        symlink: /media/${USER}

The "path" entry behaves like the tree "path" entry -- when unspecified it
defaults to a path named after the tree relative to the garden root.


## Commands


### Basics

All garden commands have this basic syntax:

    gdn [options] <command> [command-options] [command-arguments]*

The following options are specified before `<command>`, and are
global to all `garden` commands.

    -c | --config <path>

Specify a garden config file to use instead of searching for `garden.yaml`.
The path can either be the path to an actual config file, or it can be
the basename of a file in the configuration search path.

Config files found through the search path must have a `.yaml` or `.json` extension.
When specifying a search path basename, the filename can be specified either
with or without an extension.  Garden will add the extension automatically.

    -v | --verbose

Enable verbose debugging output.


    -s | --set  name=value

Override a configured variable by passing a `name=value` string to
the `--set` option.  The variable named `name` will be updated with the
garden expression `value`.  Multiple variables can be set by specifying the
flag multiple times.


#### Gardens, Groups and Trees

Trees are Git repositories with configuration that allows for the
specification of arbitrary commands and workflows.  Groups are a simple
named grouping mechanism.

Gardens can include groups and trees, as well as environment, gitconfig, and
custom commands in addition to the ones provided by each tree.

#### Tree queries

Command arguments with explicit garden names, `@tree` references,
`%group` syntax, and wildcards are all referred to as "tree queries".
Tree queries are strings that resolve to a set of trees.

Glob  wildcards in tree queries allows operations to span over ad-hoc gardens,
groups, and trees.  A garden.yaml configuration can be carefully crafted for
maximum glob-ability.

For example, `gdn exec 'annex/*' operates on all the `annex/` repositories.

Garden commands take tree query arguments that specify which trees to
operate on.  When a name is mentioned on the command-line, garden will use the
first matching garden, group, or tree, in that order, when determining which
trees to operate on.  When a garden or group is matched, all of its associated
trees are included in the operation.

Paths can be specified as well, but the filesystem has the lowest priority
relative to gardens, groups, and trees.  When specifyiing paths they must
resolve to a configured tree.  For example:

    gdn build .

Will build the tree in the current directory.


In the following example, the "cola" group is found
in the example configuration, and the commands are run over multiple repos.

    gdn exec cola git status -s
    gdn status cola
    gdn cmd cola status build

If you have groups, gardens, and trees with the same name then you can use the
`@tree`, `%group`, and `:garden` syntax to disambiguate the name.

    gdn init @tree      # initialize the tree called "tree"
    gdn init %group     # initialize the group called "group"
    gdn init :garden    # initialize the garden called "garden"

Gardens have highest priority, so the ":garden" query is rarely needed
in practice.  "%group" and "@tree" queries refer to groups and trees only, but
not gardens which have the same name.

Garden understands shell wildcards, so multiple trees, gardens, or
groups can be matched by using wildcards.  For example,

    gdn init '@annex/*'

initializes all trees that start with "annex/".


### gdn add

    gdn add <path>

    # example
    gdn add src/repo

Add an existing Git tree at `<path>` to `garden.yaml`.


### gdn cmd

    gdn cmd <query> <command>...

    # example
    gdn cmd cola build test

Run commands over the trees matched by `<query>`.

Custom commands can be defined at either the tree or garden level.
Commands defined at the garden level extend commands defined on a tree.


### garden custom sub-commands

    gdn <command> <query>...

    # example
    gdn status @cola .
    gdn build . git

When no builtin command exists by the specified name then garden will
use custom commands defined in a "commands" block at the corresponding
garden or tree scope.

This is complementary to `gdn cmd <query> <command>...`.
That form can run multiple commands; this form operates over multiple queries.


### gdn exec

    gdn exec <query> <command> <arguments>...

    # example
    gdn exec cola git status -s

Execute a command on all of the trees matched by `<query>`.


### gdn eval

    gdn eval <expression> [<tree>] [<garden>]

    # example
    gdn eval '${GARDEN_ROOT}'
    gdn eval '${TREE_PATH}' cola

Evaluate a garden expression in the specified tree context.
If no tree is given then the variable scope includes the top-level variables
block only.

When a tree is given then its variables are considered as well.
When a garden is specified then the garden's variables are also available for
evaluation.


### gdn init

    gdn init <query>

    # example
    gdn init cola

Initialize the tree(s) referenced by the `<query>`.  The `init` command can
also be used to update existing trees.  It is safe to re-run the `init`
command.  For existing trees, their git configuration will be updated to match
any changes made to the configuration.  Missing repositories are cloned from
their configured url.


### gdn shell

    gdn shell <query> [<tree>]

    # example
    gdn shell cola

Launch a shell inside the environment formed by the tree query.
The optional tree argument specifies which tree to chdir into.

If the resolved tree query contains a tree whose name exactly matches the
query string then that tree's directory will be used when opening the shell.
The optional tree argument is not needed for the common case where a garden
and tree share a name -- garden will chdir into that same-named tree when
creating the shell.
