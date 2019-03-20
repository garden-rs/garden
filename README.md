# garden -- grow and cultivate collections of Git trees

Garden makes it easy to run operations over arbitrary collections of
self-contained Git trees.


## Use cases

* Make it easy to define collections of independent Git repositories and
  group them into cohesive development gardens.
* Build/test/install interdependent projects in self-contained sandboxes.
* Bootstrap development environments from simple configuration.
* Make it easy to recreate complex Git-based development environments from
  scratch using a simple yaml configuration format.

Garden is useful when you're writing software and weaving together
development environments directly from Git trees.  Garden aids in common
development setup steps such as setting environment variables, search paths,
and creating arbitrary groupings of repositories for development.


## Configuration

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

Example `garden.yaml`:

    garden:
      root: ~/src

    variables:
      flavor: debug
      prefix: ${TREE_PATH}/target/${flavor}
      num_procs: $ nprocs 2>/dev/null || echo 4

    commands:
      status:
        - git branch
        - git status -s

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


    # wildcards can be used when specifying names in a group member list,
    # or in a garden's group and tree lists.

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


Gardens aggregate groups and trees.  Define a group and reuse the group in
each garden to share tree lists between gardens.

Templates allow sharing of variables, gitconfig, and environments between
trees.  Trees can also reuse another tree definition by specifying the
"extend" keyword with the name of another tree.  Only the first remote is used
when extending a tree.

Fields that expect lists can also specify a single string, and the list
will be treated like a list with a single item.  This is useful, for example,
when defining groups that consist of a single wildcard pattern.

The names in garden `tree` and `group` lists, and group member names accept glob
wildcard patterns.

For example, the "annex" group matches all trees that start with "annex/".
The "git-all" group has two entries -- the first matches all trees that start
with "git", and the second one matches "cola" explicitly.


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


    -s | --set  $name=$value

Override a configured variable by passing a `$name=$value` expression to
the `--set` option.  The variable named `$name` will be updated with the
expression `$value`.  This option can be specified multiple times.

#### Gardens, Groups and Trees

Garden commands take command-line arguments that specify a subset of
trees to operate on.  When a name is mentioned on the command-line, garden
will use the first matching garden, group, or tree, in that order, when
determining which trees to operate on.  When a garden or group is matched,
all of its associated trees are included in the operation.

When matching names, gardens have the highest precedence, followed by groups,
and finally trees.  In the following example, the "cola" group is found
in the example configuration, and the commands are run over multiple repos.

    gdn exec cola git status -s
    gdn status cola
    gdn cmd cola status build

If you have groups, gardens, and trees with the same name then you can use the
`@tree`, `%group`, and `:garden` syntax to disambiguate the name.

    gdn init @tree  # initialize the tree called "tree"
    gdn init %group  # initialize the group called "group"
    gdn init :garden  # initialize the garden called "garden"

Garden understands shell wildcards, so multiple trees, gardens, or
groups can be matched by using wildcards.  For example, `garden init '@x*'`
initializes all trees that start with "x".

When a tree expression matches an existing path on disk, garden will use that
path to find a matching tree in its tree configuration.  The matching tree
will be used for the command when found.


#### Tree Expressions

Command arguments with explicit garden names, @tree references, %group syntax,
and wildcards are all referred to as "tree expressions".  Tree expressions
are strings that resolve to a set of trees.


### gdn add

    gdn add <path>

Add an existing tree at `<path>` to `garden.yaml`.


### gdn cmd

    gdn cmd <tree-expression> <command>*

Run command(s) over the trees matched by `<tree-expression>`.
For example, to build and test Git, `gdn cmd git build test`.

Custom commands can be defined at either the tree or garden level.
Commands defined at the garden level extend commands defined on a tree.


### garden custom sub-commands

    gdn <command> <tree-expression>*

    gdn diff @cola

When no builtin command exists by the specified name then garden will
run custom commands by that name.

This is complementary to `gdn cmd <tree-expression> <command>*`
because that form allows multiple commands; this form allows multiple
tree-expressions, and is convenient to type.


### gdn exec

    gdn exec <tree-expression> <command> <arguments>*

Execute a command on all of the trees matched by `<tree-expression>`.
Example: `gdn exec cola git status -s`.


### gdn eval

    gdn eval <expression> <tree> [<garden>]

Evaluate a garden expression in the specified tree context.


### gdn init

    gdn init <tree-expression>

Initialize the tree(s) referenced by the `<tree-expression>`.
Garden will use the first set of matching trees, gardens, or groups, in that
order, when determining which trees to initialize.

The `init` command can also be used to update existing trees.  It is always
safe to re-run the `init` command.  For existing trees, their git
configuration will be updated to match any changes made to the configuration.


### gdn shell

    gdn shell <tree-expression> [<tree>]

Launch a shell inside the environment formed by the tree expression.
The optional tree argument specifies which tree to chdir into.

If the resolved tree expression contains a tree whose name exactly matches the
expression itself that tree's directory will be used when opening the shell.


## Variables

Garden configuration contains a "variables" block that allows defining
variables that are can be referenced by other garden values.

    variables:
        flavor: debug
        home: ~
        user: $ whoami
        libdir: $ test -e /usr/lib64 && echo lib64 || echo lib
        prefix: ${TREE_PATH}/target/${flavor}
        py_ver_code: from sys import version_info as v; print("%s.%s" % v[:2])
        py_ver: $ python -c '${py_ver_code}'
        py_site: ${libdir}/python${py_ver}/site-packages

Variables definitions can reference environment variables and garden variables
that were defined before them a "variables" block.

Variable references use shell `${variable}` syntax.

Values that start with dollar-sign, space (`$ `) are called "exec expressions".
Exec expressions are run through a shell after evaluation and replaced with
the output of the evaluated command.

Garden variables references and Exec expressions can be used within tree and
garden definitions.

When resolving values, variables defined in a tree scope override/replace
variables defined at the global scope.  Variables defined in garden scope
override/replace variables defined in a tree scope.

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
their name, for example `${TREE_NAME}_LOCATION=: ${TREE_PATH}` exports a
variable called `xyz_LOCATION` with the location of the `xyz` tree when used
inside the `xyz` tree's environment definition.


### OS Environment Variables

OS-level environment variables that are present in garden's runtime
environment can be referenced using garden `${variable}` expression syntax.
Garden variables have higher precedence than environment variables when
resolving `${variable}` references -- the environment is checked only when
no garden variables exist by that name.
