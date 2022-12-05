# Changelog

## v0.4.0

**Features**:

- `garden prune` was added for removing orphaned Git repositories.
  ([#4](https://github.com/davvid/garden/issues/4))

- `garden cmd` learned to run commands in breadth-first order when the
  `-b/--breadth-first` option is used. Depth-first tree traversal is the default.
  The `garden cmd --breadth-first` traversal runs all commands on a tree before
  continuing on to the next tree. The default `garden cmd` depth-first traversal
  runs a command across all trees before continuing on to the next command.
  ([#3](https://github.com/davvid/garden/issues/3))

## v0.3.0

**Features**:

- `garden plant` learned to detect `git worktree` repositories.
  ([#1](https://github.com/davvid/garden/issues/1))

## v0.2.0

**Breaking Changes**:

- `garden add` was renamed to `garden plant`.

**Features**:

- `garden grow` learned to grow trees using "git worktree" (#1).
- `garden grow` learned to clone specific branches.
- `garden grow` and `garden plant` learned to handle bare repositories.


## v0.1.0

**Features**:

This is the initial garden release.

- `garden grow` grows worktrees.
- `garden init` intitializes configuration.
- `garden plant` (formerly `garden add`) adds existing trees.
- `garden cmd` and `garden <custom-command>` can run custom commands.
- Templates, variables, and environment variables are all supported.
