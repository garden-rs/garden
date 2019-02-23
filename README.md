# garden -- grow and cultivate collections of Git trees

Garden makes it easy to run operations over arbitrary collections of
self-contained Git trees.


## Use cases

* Make it easy to define collections of independent Git repositories and
  group them into cohesive development gardens.
* Build/test/install interdependent projects in self-contained sandboxes.
* Bootstrap development environments from simple configuration.
* Make it easy to configure a Git-based development environment so that it can
  be recreated from scratch.

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
    groups:
        cola: [cola, git, qtpy, vx]
    gardens:
        cola:
            groups: cola
            commands:
                init:
                    - cd ${cola_PATH}
                    - virtualenv --system-site-packages env27
                    - vx env27 make requirements
                    - vx env27 make requirements-dev
                build: cd "${cola_PATH}" && vx env27 make all
                doc: cd "${cola_PATH}" && vx env27 make doc
                test: cd "${cola_PATH}" && vx env27 make test
                run: vx "${cola_PATH}/env27" git cola
        gitdev:
            variables:
                prefix: ~/apps/gitdev
            environment:
                PATH: ~/apps/gitdev/bin
            trees: git, gitk

    templates:
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
            packages:
                apt: python-qt5
        garden:
            url: git://github.com/davvid/garden.git
            templates: rust
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


## Commands


### Basics

All garden commands have this basic syntax:

    garden [options] <command> [command-options] [command-arguments]*

The following options must be specified before `<command>`, and is
global to all `garden` commands.

    -c | --config <path>

Specify a garden config file to use instead of searching for `garden.yaml`.

    -v | --verbose

Enable verbose debugging output.


#### Gardens, Groups and Trees

Garden commands take command-line arguments that specify a subset of
trees to operate on.  When a name is mentioned on the command-line, garden
will use the first matching garden, group, or tree, in that order, when
determining which trees to operate on.  When a garden or group is matched,
all of its associated trees are included in the operation.

When matching names, gardens have the highest precedence, followed by groups,
and finally trees.

If you have groups, gardens, and trees with the same name then you can use the
`@tree`, `%group`, and `:garden` syntax to disambiguate the name.

    garden init @tree  # initialize the tree called "tree"
    garden init %group  # initialize the group called "group"
    garden init :garden  # initialize the garden called "garden"

Garden understands shell wildcards, so multiple trees, gardens, or
groups can be matched by using wildcards.  For example, `garden init '@x*'`
initializes all trees that start with "x".


#### Tree Expressions

Command arguments with explicit garden names, @tree references, %group syntax,
and wildcards are all referred to as "tree expressions".  Tree expressions
are strings that resolve to a set of trees.


### garden init

    garden init <tree-expression>

Initialize the tree(s) referenced by the `<tree-expression>`.
Garden will use the first set of matching trees, gardens, or groups, in that
order, when determining which trees to initialize.

The `init` command can also be used to update existing trees.  It is always
safe to re-run the `init` command.  For existing trees, their git
configuration will be updated to match any changes made to the configuration.


### garden add

    garden add <path>

Add the tree at `<path>` to `garden.yaml`.


### garden exec

    garden exec <tree-expression> <command> <arguments>*

Execute a command on all of the trees matched by `<tree-expression>`.
Example: `garden exec cola git status -s`.


### garden eval

    garden eval <expression> <tree> [<garden>]

Evaluate a garden expression in the specified tree context.


### garden cmd

    garden cmd <tree-expression> <command>*

Run command(s) over the trees matched by `<tree-expression>`.
For example, to build and test Git, `garden cmd git build test`.

Custom commands can be defined at either the tree or garden level.
Commands defined at the garden level extend commands defined on a tree.


### garden custom sub-commands

    garden <command> <tree-expression>*

    garden diff @cola

When no builtin command exists by the specified name then garden will
run custom commands by that name.

This is complementary to `garden cmd <tree-expression> <command>*`
because that form allows multiple commands; this form allows multiple
tree-expressions, and is convenient to type.


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


### OS Environment Variables

OS-level environment variables that are present in garden's runtime environment
are pre-populated into garden's variables.   This allows for `${variable}`
expressions to reference values from garden's runtime environment.

To disable loading of environment variables, configure the
`garden.environment_variables` value to `false`.
