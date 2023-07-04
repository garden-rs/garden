# Glossary

* ***garden.yaml*** -- a Garden file defines trees, groups gardens,
  variables, commands and environment variables.

* ***tree*** -- a Git worktree. A single repository clone.
  A tree represents a single repository and its configuration.
  Trees provide garden variables, commands and environment variables that
  are defined when commands are run on a tree. Environment variables are
  defined when a tree is operated on by itself and when combined alongside
  other trees in a group or garden context.

* ***group*** -- a named collection of trees. The "group" block in the
  configuration file defines groups of trees that are available for use by
  commands and gardens. Groups are lighter-weight than a garden. They group
  trees together into named collections that can be referenced by gardens
  and commands. In contrast to gardens, groups *do not* provide a scope
  in which variables, commands environment variables can be defined.

* ***garden*** -- a composite of trees and groups for defining project-level
  variables, commands and environment variables. Gardens help create environments
  for working on libraries, and application that use the library, in tandem.
  The `gardens` YAML key/value block defines named gardens with a scope
  in which garden-specific variables, commands and environment variables
  can be defined.

* **variables**
  A YAML key/value table defines named variables that can be used in
  YAML strings by using braced `shell ${expressions}` to provide dynamic
  values that can be used to provide modularity and configurability to a garden
  file. Variables defined in a garden file are overridden on the
  command-line by passing `-D / --define key=value`.

* **commands**
  The `commands` YAML key/value table defines named commands that can
  be run against trees. Commands are used to extend the `garden` command
  itself with user-defined functionality. The `commands` block can be
  defined at global scope, within a `tree` block, and within a `garden` block.
  The scope in which a command is defined limits the scope in which it
  executes. This means that a command defined in tree scope will only
  execute when `garden <command> <query>` results in a query that ends
  up including that tree.

* **environment variables**
  Not to be confused with the `variables` block, the `environment` block
  is a YAML key/value table that defines environment variables that will
  be set when executing custom commands. String values can use `${variables}`
  and *exec expressions* when defining the environment variable values.

* ***tree query*** -- a [shellexpand] glob expression that matches against
  garden names, group names and tree names. Several `garden` builtin commands
  take tree queries as arguments as a mechanism for selecting one or more
  trees to operate on. The default behavior is to match the tree query pattern
  against gardens, groups and trees, in that order, and return the first
  matching set of trees. If searching garden finds matches then groups and
  trees are not searched. If searching group finds matches then trees are not
  searched. Prefix the tree query pattern with a percent-sign (`%`), eg. `%group*`,
  to only query for groups matching the pattern. Prefix the pattern with an
  at-sign (`@`), eg. `@tree`, to only query for trees. Prefix the pattern
  with a colon (`:`), eg. `:garden`, to only query for gardens. The `:garden`
  syntax is not typically used because gardens are already searched first
  by default; `%group` and `@tree` can be useful when disambiguating
  trees, groups and gardens that happen to share the same name.

* **exec expressions**
  When a variable or any string value starts with `$ ` (dollar-space)
  then the variable's value is computed by expanding the string's
  `${garden}` variables and then executing the result and capturing stdout.
  The captured stdout output from the command becomes the variable's value.
  For example, if a variable called `my-pwd` is defined using an exec expression
  such as `my-pwd: $ pwd` then the `${my_pwd}` variable will contain a path.
  The `my_pwd` variable can be used to define other variables,
  environment variables, and commands. For example, a command called `example-cmd`
  can be defined as follows: `example-cmd: ls -l ${my-pwd} && echo my_pwd is ${my_pwd}`,

Gardens can be used for software projects composed of multiple repositories,
or they can be used to provide an approachable way to explore a curated set of
Git repositories for any purposes.

[shellexpand]: https://github.com/netvl/shellexpand
