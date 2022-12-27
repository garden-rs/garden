# Changelog

## v0.4.1

**Features**

- The `garden cmd --no-errexit` option was extended to work with commands that are
  configured using a YAML list of strings. Commands that are specified using lists
  are now indistinguishable from commands specified using multi-line strings.

## v0.4.0

**Breaking Changes**:

- `garden cmd` now runs custom commands using `<shell> -e -c '<command>'` by default.
  The use of `-e` is a change in behavior and causes multi-line / multi-statement
  commands to halt execution when the first non-zero exit code is encountered.
  Use `set +e` at the top of of a multi-line command to opt-out of this behavior
  on a per-command basis, or specify the `-n | --no-errexit` option.

- `garden` will now fallback to `bash` (and `sh`) as the default `garden.shell` value
  when `zsh` (and `bash`) are not installed. As before, the `garden.shell`
  configuration variable can be used to override the default shell.

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
