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

If `garden.yaml` is not found in these directories then `garden` will walk up the
file system searching for config files.

You can prevent `garden` from traversing into directories higher in the file system
tree by setting either the `GARDEN_CEILING_DIRS` or `GIT_CEILING_DIRS` environment
variables. Multiple directories can be specified by using a colon(`:`)-delimited
list of ceiling directories.

`garden` uses the [GIT_CEILING_DIRS](https://git-scm.com/book/en/v2/Git-Internals-Environment-Variables)
environment variable from core Git as a fallback when `GARDEN_CEILING_DIRS` it is not set.

`GARDEN_CEILING_DIRS` has higher precedence and overrides values configured in
`GIT_CEILING_DIRS`.

Use `garden -c|--config <filename>` to specify a garden file and override
`garden`'s file discovery.

If a basename is specified, e.g. `garden --config custom.yaml`, then `garden` will search
these same locations for `custom.yaml` instead of `garden.yaml`.

The following example `garden.yaml` is referred to by the documentation
when showing examples.

```yaml
{{#include examples/git-cola/garden.yaml}}
```


## Garden Root

The garden root directory is configured in the `garden.root` field.
This directory is the parent directory in which all trees will be cloned.

`garden.root` defaults to `${GARDEN_CONFIG_DIR}` when unspecified.
`{GARDEN_CONFIG_DIR}` is the directory that contains the current `garden.yaml` .

The default `${GARDEN_CONFIG_DIR}` value lets you create relocatable setups
with trees that are located relative to the `garden.yaml` config file.

Slashes in tree paths will create new directories on disk as needed.

```yaml
# By default, "garden init" generates a "garden.yaml" that uses the
# ${GARDEN_CONFIG_DIR} for its "garden.root" location.
garden:
  root: ${GARDEN_CONFIG_DIR}
```

Omitting `garden.root` is equivalent to the configuration above.

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

Configure `garden.root` to `""` (an empty string) to use a dynamic
`garden.root` that follows your current directory. This lets you use
`garden --config <path>` to create and interact with trees in your
current directory instead of a fixed configuration-defined location.

```yaml
garden:
  root: ""
```

## Garden Shell

The shell used by `garden` when running Commands is configured by the `garden.shell` field.

```yaml
garden:
  shell: zsh
```

The `$PATH` environment variable is probed for available shells in the following order
when `garden.shell` is omitted. The first one found is the one that's used.

* `zsh`
* `bash`
* `dash`
* `sh`

The following shell interpreters can also be configured in `garden.shell`
for running custom commands.

* `bun`
* `fish`
* `node`
* `perl`
* `python3`

Any command that can run command strings using `-c` can be used as a
`garden.shell` interpreter.

Exec expressions are always run using the default `/bin/sh` system shell
irrespective of `garden.shell`.

## Tree Display

Garden will display the tree's current branch when running commands.
While this has a marginal performance impact, this feature can be disabled by either
passing the `garden -D garden.tree-branches=0` option or by configuring the
`garden.tree-branches` option to `false` in the `garden.yaml` configuration.

```yaml
garden:
  tree-branches: false
```

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
found are silently ignored.

Enable the `garden -d config ...` debug flag to display warnings about missing include
files.


### The "Last One Wins" Rule

Entities in garden files such as `trees`, `gardens`, `groups`, `commands` and
`variables` can be sparsely defined across multiple files by using `includes`.

When the same entry is found in multiple included files then the only last definition
will be used. This is referred to as the "Last One Wins" rule.

Entities defined in the root `garden.yaml` have the highest precedence and override
entries provided via `garden.includes`.

`variables`, `commands`, `groups` and `gardens` are completely replaced when multiple
definitions are found.

`trees` are sparsely overridden. If an override definition in the top-level
`garden.yaml` replaces just the `url` field, for example, then all of the `commands`,
`variables` and `environment` values from the earlier definition are retained
and only the `url` for the `origin` remote is replaced.

If a tree needs to completely override a base definition then a tree can use
`replace: true` to indicate that the tree definition is replacement for the
earlier tree definition.

```yaml
# garden.yaml
---
garden:
  includes: trees.yaml

trees:
  example:
    replace: true
    url: https://custom.example.com/custom/tree
...
```

The `garden.yaml` above includes `trees.yaml`. The `example` tree is originally
defined here, but it is completely replaced by the same entry above.

```yaml
# trees.yaml
---
trees:
  example: https://example.com/original/tree
  commands:
    echo: Hello, ${TREE_NAME}
...
```

In the example above, the `example` tree completely replaces the
same tree from the included `trees.yaml`. None of the commands, variables or other
settings from the replaced tree are retained.


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
  py_ver: $ python3 -c '${py_ver_code}'
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

* **GARDEN_CONFIG_DIR** -- Directory containing the `garden.yaml` file.
* **GARDEN_ROOT** -- Root directory for trees.
* **TREE_NAME** -- Current tree name.
* **TREE_PATH** -- Current tree path.

## Environment Variables

The "environment" block defines variables that are stored in the environment.

Environment variables are resolved in the same order as garden variables:
global scope, tree scope, and garden scope.  This allows gardens to
prepend/append variables after a tree, thus allowing for customization
of behavior from the garden scope.

Values in environment blocks prepend to the environment variable by default.
The `:` UNIX path separator is used when prepending and appending values.

```yaml
trees:
  foo:
    environment:
      PATH: ${TREE_PATH}/bin
```

The example above prepends the `foo/bin` directory to the colon (`:`)-delimited `PATH`
environment variable.

Names with an equals sign (`=`) suffix are treated as "store" operations and are
stored into the environment, fully replacing any pre-existing values.

```yaml
trees:
  foo:
    environment:
      ${TREE_NAME}_LOCATION=: ${TREE_PATH}
```

Environment variable entries can use garden `${variable}` syntax when defining
both their name and values. The example above exports a variable called `foo_LOCATION`
with the location of the tree. If `foo_LOCATION` is already defined then its value is
replaced.

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


## Trees

The `trees` block in a `garden.yaml` Garden file defines the trees that
are available for running commands against.

Trees represent Git worktrees and can be aggregated into groups and gardens.
The `tree` block defines properties about a tree such as its Git URL,
custom variables, environment variables, Git remotes, Git configuration variables,
and custom commands.

```yaml
trees:
  git-scm:
    description: Fast, scalable, distributed version control
    path: git
    url: git://git.kernel.org/pub/scm/git/git.git
    remotes:
      gitlab: https://gitlab.com/gitlab-org/git.git
      github: https://github.com/git/git.git
      gitster: https://github.com/gitster/git.git
    commands:
      build: make all -j ${num_procs} "$@"
      test: make test "$@"
    variables:
      num_procs: $ nproc 2>/dev/null || sysctl -n hw.activecpu 2>/dev/null || echo 4
```

All fields are more or less optional. The `path` field defaults to the same
name as the tree, so the `path: git` setting above can be omitted without
any differences in behavior.

The `path` field can be used to name trees independently of directory on disk.
The `path` value defaults to the same name as the tree, for example `git-scm`
would be used as the directory without the `path: git` setting above.

The `path: git` setting causes `garden grow git` to clone into a directory called
`git` instead of `git-scm`.

Relative paths are assumed to be relative to the `${GARDEN_ROOT}`, typically the
same directory as the `garden.yaml`.

If the only field you want to configure is the `url` field then you can also
use a `string` rather than a `dictionary` / `table` to represent a tree
by providing just the URL. This configures a tree with a single `origin`
remote pointing to the configured URL.

```yaml
trees:
  git-scm: git://git.kernel.org/pub/scm/git/git.git
```

### Default Tree

The `trees` block is optional when a `commands` block exists.

An implicit default tree called `.` will be synthesized into existence when the `trees`
block is empty.

The default tree's `path` is set to the `${GARDEN_CONFIG_DIR}`.
Omitting the `trees` block lets you use `garden` as a simple command runner.

### Remotes

The `remotes` field defines named
[Git remotes](https://git-scm.com/book/en/v2/Git-Basics-Working-with-Remotes)
that will be configured when growing the repository.

If you edit your `garden.yaml` then you can always re-run `garden grow` to add remotes
that were added to `garden.yaml`.

Likewise, you can always rerun `garden plant` to record new remotes that
you may have added using the `git remote add`
[command-line interface](https://git-scm.com/docs/git-remote).

### Default Remote Name

The default `origin` remote name used by Git can be overridden by setting the
`default-remote` field in the tree's configuration.

```yaml
trees:
  git:
    url: git://git.kernel.org/pub/scm/git/git.git
    default-remote: kernel.org
```

This will create a remote called `kernel.org` instead or `origin` when growing trees.
This feature can also be used when multiple named remotes are configured.

```yaml
trees:
  git:
    default-remote: kernel.org
    remotes:
      kernel.org: git://git.kernel.org/pub/scm/git/git.git
      gitster: https://github.com/gitster/git.git
```


## Templates

Templates allow sharing of command, variable, gitconfig, and environment
definitions across trees. Adding an entry to the `templates` configuration block
makes a template available when defining trees.

Specify `templates: <template-name>` to inherit the specified template's
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

When a tree specifies multiple templates then all of the templates are merged into
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
`cola`.  The first matches all trees that start with "git", and the second one
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


## Grafts

A more advanced modularity feature allow you to stitch additional `garden.yaml`
files underneath a custom "graft namespace".

The example below demonstrates how to define trees and variables in separate
"graft" files and refer to them using a `graft::` namespace qualifier.

```yaml
# Top-level garden.yaml
grafts:
  graft: graft.yaml
  graft-repos:
    config: repos.yaml
    root: repos

trees:
  local-tree:
    url: https://git.example.com/repo.git
    variables:
      value: "local ${graft::value}"

variables:
    value: "global ${graft::value}"

gardens:
  example:
    trees:
      - local-tree
      - graft::tree
      - graft-repos::example
```

The `graft-repos` graft entry demonstrates how to use a custom root directory for
the trees provided by the grafted configuration.

The `grafts.yaml` file provides a tree called `tree` and a variable called `value`.
We refer to them as `graft::tree` when specifying trees and `${graft::value}` when
using variables.

`graft.yaml` contains the following:

```yaml
# The grafted "graft.yaml" file.
trees:
  tree: https://git.example.com/tree.git

variables:
  value: "grafted value"
```

Running `garden eval '${graft::value}'` will output `grafted value`.

Running `garden eval '${value}'` will output `global grafted value`, as it evaluates at
global scope.

Running `garden eval '${value}' local-tree` will output `local grafted value`, as it
evaluates from `local-tree`'s scope.
