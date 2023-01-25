# Configuration

Garden is configured through a YAML configuration file, typically called
"garden.yaml".

Garden will find `garden.yaml` in current directory or in specific locations
on the filesystem when unspecified.  Garden searches for `garden.yaml` in the
following locations. The first one found is used.

```sh
# Relative to the current directory
./garden.yaml
./garden/garden.yaml
./etc/garden/garden.yaml

# Relative to $HOME
~/.config/garden/garden.yaml
~/etc/garden/garden.yaml

# Global configuration
/etc/garden/garden.yaml
```

Use `garden -c|--config <filename>` to specify a garden file and override
garden's file discovery.

The following example `garden.yaml` is referred to by the documentation
when showing examples.

```yaml
{{#include examples/git-cola/garden.yaml}}
```


## Garden Root

The garden root directory is configured in the `garden.root` field.
This directory is the parent directory in which all trees will be cloned.

Slashes in tree paths will create new directories on disk as needed.
`garden.root` defaults to the current directory when unspecified.

The built-in `${GARDEN_CONFIG_DIR}` variable can be used to create relocatable
setups that define a `garden.root` relative to the garden file itself.

To place all trees in a `src` directory sibling to the `garden.yaml` file, the
following configuration can be used:

```yaml
garden:
  root: ${GARDEN_CONFIG_DIR}/src
```

To place all trees in a `src` directory in your `$HOME` directory, the
following configuration can be used:

```yaml
garden:
  root: ~/src
```

## Garden Shell

The shell used by `garden` when running Commands and evaluating Exec Expressions
can be configured in the `garden.shell` field.

```yaml
garden:
  shell: zsh
```

The `$PATH` environment variable is probed for available shells in the following order
when `garden.shell` is omitted. The first one found is the one that's used.

* `zsh`
* `bash`
* `sh`


## Includes

Garden files can be split apart into several files for modularity and reuse.
You can use the `garden.includes` list to specify other garden files to include
into the current garden file.

```yaml
garden:
  includes:
    # Includes are relative to the GARDEN_CONFIG_DIR by default.
    - variables.yaml
    # Includes can reference custom and built-in ${variables}.
    - ${include_dir}/commands.yaml

variables:
  include_dir: ${GARDEN_ROOT}
```

Absolute paths in the `garden.includes` list are included as-is.
Relative paths in the `garden.includes` list are resolved.

Relative paths are first resolved relative to the file in which they are defined.
This allows nested include files to use relative paths for their nested includes.

If an include file is not found relative to the current garden file then a path relative
to the root configuration directory will be checked for the existence of the file.

Includes files are treated like "optional" includes -- include files that cannot be
found are silenty ignored.

Enable the `garden -d config ...` debug flag to display warnings about missing include
files.

The top-level `commands` block follows "last one wins" semantics. If the same command
is defined in multiple include files then the last definition is the one that will
be used. `commands` defined in the the root `garden.yaml` have the highest precedence
and override commands defined via `garden.includes`.


## Variables

Garden configuration contains a "variables" block that allows defining
variables that are can be referenced by other garden values.

```yaml
variables:
  flavor: debug
  user: $ whoami
  libdir: $ test -e /usr/lib64 && echo lib64 || echo lib
  nproc: $ nproc
  prefix: ~/.local
  py_ver_code: from sys import version_info as v; print("%s.%s" % v[:2])
  py_ver: $ python -c '${py_ver_code}'
  py_site: ${libdir}/python${py_ver}/site-packages
```

Variables definitions can reference environment variables and other garden
variables.

Variable references use shell `${variable}` syntax.

Values that start with dollar-sign+space (`$ `) are called "exec expressions".
Exec expressions are run through a shell after evaluation and replaced with
the output of the evaluated command.

When resolving values, variables defined in a tree scope override/replace
variables defined at the global scope.  Variables defined in garden scope
override/replace variables defined in a tree scope.


## Built-in variables

Garden automatically defines some built-in variables that can be useful
when constructing values for variables, commands, and paths.

* **GARDEN_CONFIG_DIR** -- Directory containing the `garden.yaml` config file.
* **GARDEN_ROOT** -- Root directory for trees.
* **TREE_NAME** -- Current tree name.
* **TREE_PATH** -- Current tree path.

## Environment Variables

The "environment" block defines variables that are stored in the environment.

Environment variables are resolved in the same order as the garden variables:
global scope, tree scope, and garden scope.  This allows gardens to
prepend/append variables after a tree, thus allowing for customization
of behavior from the garden scope.

Environment variables are resolved after garden variables.  This allows
the use of garden variables when defining environment variable values.

Environment variable names can use garden `${variable}` syntax when defining
both their name and values.

Values in environment blocks prepend to the named environment variable.

```yaml
trees:
  foo:
    environment:
      PATH: ${TREE_PATH}/bin
```

The example above prepends the `foo/bin` directory to the colon (`:`)-delimeted `PATH`
environment variable.

Names with an equals sign (`=`) suffix are treated as "store" operations and are
stored into the environment, fully replacing any pre-existing values.

```yaml
trees:
  foo:
    environment:
      ${TREE_NAME}_LOCATION=: ${TREE_PATH}
```

The example above exports a variable called `foo_LOCATION` with the location of the tree.
If `foo_LOCATION` is already defined then it its value is replaced.

A plus sign (`+`) suffix in the name append to a variable instead of prepending.

```yaml
trees:
  foo:
    environment:
      PATH+: ${TREE_PATH}/bin
```

The example above appends to the `PATH` environment variable.
Note the `+` suffix after `PATH`.


### OS Environment Variables

OS-level environment variables that are present in garden's runtime
environment can be referenced using garden `${variable}` expression syntax.
Garden variables have higher precedence than environment variables when
resolving `${variable}` references -- the environment is checked only when
no garden variables exist by that name.


## Gardens, Groups and Trees

Trees are Git repositories with configuration that allows for the
specification of arbitrary commands and workflows.  Groups are a simple
named grouping mechanism.

Gardens aggregate groups and trees.  Define a group and reuse the group in
different gardens to share tree lists between gardens.  Defining gardens
and groups make those names available when querying and performing operations
over trees.

Gardens can also include environment, gitconfig, and custom group-level
commands in addition to the commands provided by each tree.


## Templates

Templates allow sharing of command, variable, gitconfig, and environment
definitions across trees. Adding an entry to the `templates` configuration block
makes a template available when defining trees.

Specify `templates: <template-name>` to inherit the the specified template's
settings when defining trees. The `templates` field also accepts a list of
template names.

Trees can also reuse another tree definition by specifying the "extend"
keyword with the name of another tree.  Only the first remote is used when
extending a tree.

```yaml
templates:
  hello:
    variables:
      message: Hello ${TREE_NAME}.
    commands:
      echo: echo ${message}

trees:
  hello-tree:
    templates: hello

  hello-tree-extended:
    extend: hello-tree
    variables:
      message: The time is now: $(date)
```

When a tree specifies multiple templates then all of the tempaltes are merged into
the tree's definitions. If variables are multiply-defined across multiple templates
then the variable's value from the last specified template will be used.


## String to List Promotion

Fields that expect Lists can also be specified using a String value.
Strings will be promoted to a List containing a single String.
This is useful when defining `commands` and `groups`.

The `commands` block defines commands that are specified using Lists of Strings.
_String to List Promotion_ makes it easier to define commands by specifying a single
String that can either be a simple value or a multi-line YAML String.

The following commands show the various ways that `commands` can be specified
due to the automatic promotion of Strings into Lists.

```yaml
commands:
  # commands are a list of command strings.
  cmd1:
    - echo ${TREE_NAME}
    - pwd

  # strings are promoted to a list with a single item.
  cmd2: echo ${TREE_NAME} && pwd

  # cmd2 is promoted into
  cmd2:
    - echo ${TREE_NAME} && pwd

  # multi-line command strings are supported using "|" YAML syntax.
  cmd4: |
    echo ${TREE_NAME}
    pwd

  # cmd4 is promoted into
  cmd4:
    - "echo ${TREE_NAME}\npwd"
```

## Wildcards

The names in garden `tree` and `group` lists, and group member names accept glob
wildcard patterns.

The "annex" group definition is: `annex/*`.   This matches all trees that
start with "annex/".  The "git-all" group has two entries -- `git*` and
`cola`.  the first matches all trees that start with "git", and the second one
matches "cola" only.


## Symlinks

Symlink trees create a symlink on the filesystem during `garden init`.
`garden exec`, and custom `garden cmd` commands ignore symlink trees.

```yaml
trees:
  media:
    path: ~/media
    symlink: /media/${USER}
```

The "path" entry behaves like the tree "path" entry -- when unspecified it
defaults to a path named after the tree relative to the garden root.
