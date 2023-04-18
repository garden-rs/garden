# Glossary

* ***garden.yaml*** -- a Garden file defines trees, groups and gardens.

* ***tree*** -- a Git worktree. A single repository clone.

* ***group*** -- a named collection of trees. The "group" block in the
  configuration file defines tree groups that are available for use by
  commands and gardens.

* ***garden*** -- a composite of trees and groups for defining project-level
  variables and commands.  Gardens help create environments for working on
  libraries, and application that use the library, in tandem.

* ***tree query*** -- a [shellexpand] glob expression that matches against garden
  names, group names and tree names. Many `garden` commands accept tree
  queries as arguments.

Gardens can be used for software projects composed of multiple repositories,
or they can be used to provide an approachable way to explore a curated set of
Git repositories for any purposes.

Gardens allow you to define environment variables and workflow commands that
are either available to any tree or scoped to specific gardens only.

Groups are lighter-weight than a garden. They group trees together into named
collections that can be referenced by gardens and commands.

[shellexpand]: https://github.com/netvl/shellexpand
