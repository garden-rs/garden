# Changelog

## v1.1.0

**Features**

- `garden ls` now displays information about trees, groups, gardens and commands.

- `garden ls -c` (i.e. `--no-commands`) hides command details from the output.

**Development**:

- Garden is now using
  [shellexpand v3](https://gitlab.com/ijackson/rust-shellexpand#version-300-2022-12-01).

## v1.0.0

**Features**:

- Commands can now specify pre-commands and post-commands that are run before/after
  the specified command.
  ([#3](https://github.com/davvid/garden/issues/3))
  ([documentation](https://garden-rs.gitlab.io/commands.html#pre-and-post-commands))

- The default `origin` remote name can now be configured using `tree.<tree>.default-remote`.
  ([#16](https://gitlab.com/garden-rs/garden/-/issues/16))

- Commands now display the tree's current branch alongside the tree name.
  ([#18](https://github.com/davvid/garden/issues/18))

- `garden -vv exec` and `garden -vv shell` now display the command being run.

**Packaging**:

- `garden` can now be installed as a `nix flake` package.
  A `flake.nix` file is now provided.
  ([#16](https://github.com/davvid/garden/issues/16))

## v0.9.1

**Fixes**:

- `garden -D name=value` now overrides variables in all scopes.
  Variables defined in tree scope were not subject to overrides and
  will now get properly overridden by the `--define` / `-D`
  command-line options.

## v0.9.0

**Features**:

- `garden grow` now sets `git config remote.$name.tagopt --no-tags`
  when adding additional remotes. This prevents accidentally fetching tags
  when interacting with remotes.

## v0.8.1

**Fixes**:

- `garden grow` was fixed to detect existing branches when growing
  `git worktree`-created child worktrees.

**Development**:

- `strum` is now used to implement `FromStr` for `enum ColorMode`.

- [is-terminal](https://crates.io/crates/is-terminal) is now used instead of
  the unmaintained `atty` crate.

## v0.8.0

**Features**:

- `garden` now supports a `grafts` feature that allows you to
[stitch configuration entities from separate garden files](https://garden-rs.gitlab.io/configuration.html#grafts)
into a graft-specific namespace. Trees and variables from grafted configurations can be
referenced using `graft::` namespace qualifiers.

- `garden grow` can now configure [upstream branches](https://garden-rs.gitlab.io/commands.html#upstream-branches).

- `garden grow` can now configure [gitconfig settings with multiple values](https://garden-rs.gitlab.io/commands.html#upstream-branches#git-configuration-values)
  using [`git config --add <name> <value>`](https://git-scm.com/docs/git-config#Documentation/git-config.txt---add).

## v0.7.0

**Features**:

- Trees, Groups, Gardens and Commands defined in the top-level `garden.yaml` can now
  override entries defined via `garden.includes`. Configuration entities now follow
  "last one wins" semantics -- if the same entity is defined in multiple includes files
  then only the final definition will be used.
  ([#14](https://github.com/davvid/garden/issues/14))
  ([#15](https://github.com/davvid/garden/pull/15))

- [Trees now sparsely override existing entries](https://garden-rs.gitlab.io/configuration.html#l#the-last-one-wins-rule).
  This behavior allows a tree definition to replace just the `url` field, or to replace
  individual tree commands while retaining the rest. Use `replace: true` in a Tree
  definition in order to completely replace the existing entry instead of sparsely
  overriding it.

- Improved shell completions for `garden`, `garden init` and `garden plant`.

**Packaging**:

- [0323pin](https://github.com/0323) packaged `garden` for pkgsrc/NetBSD and
  [merged the package into the main branch!](http://mail-index.netbsd.org/pkgsrc-changes/2023/01/22/msg267560.html)
  ([#13](https://github.com/davvid/garden/issues/13))

## v0.6.0

*Released 2023-01-20*

**Features**:

- Both names and values in `gitconfig` can now use `${var}` expressions.
  Previously only values were evaluated. Names are evaluated now as well.

**Fixes**:

- The `zsh` workaround for `garden completion zsh` is no longer needed.
  The [documentation for generating zsh completions](https://garden-rs.gitlab.io/commands.html#zsh)
  has been updated.
  ([#10](https://github.com/davvid/garden/issues/10))

## v0.5.1

*Released 2023-01-15*

**Fixes**:

- Exec expressions were previously run with the current directory set to the
  directory from which garden was run. Exec expressions are now run in the
  tree's current directory.

## v0.5.0

*Released 2023-01-12*

**Features**:

- [Garden configuration files can now include other configuration files
  ](https://garden-rs.gitlab.io/configuration.html#includes) by specifying
  the additional files to include in the `garden.includes` field.
  The `includes` feature makes it possible to create modular and reusable garden files.
  The `trees`, `variables`, `commands`, `groups` and `gardens` defined in the included
  files are added to the current configuration.
  ([#7](https://github.com/davvid/garden/pull/7))

- [Garden commands can now reference \$shell variables](https://garden-rs.gitlab.io/commands.html#shell-syntax)
  using the standard (brace-less) shell `$variable` syntax. The braced `${garden}`
  variable syntax remains reserved for resolving Garden Variables.
  Double-`$` braces (ex: `$${...}`) can be used to escape a `$${variable}` from
  evaluation so that a literal `${variable}` value is used in the garden command.
  ([#11](https://github.com/davvid/garden/issues/11))
  ([#12](https://github.com/davvid/garden/pull/12))

- A new `garden completion` subcommand was added for providing shell command-line
  completion using the [clap_complete](https://crates.io/crates/clap_complete) crate.
  ([#9](https://github.com/davvid/garden/pull/9))

- `garden -V | --version` was added alongside the `clap` rewrite for displaying
  the `garden` command version.

**Development**:

- The `Makefile` has been replaced by a `garden.yaml` Garden file.
  We can now use `garden {build, test, check, fmt, clippy, ...}` instead of `make ...`.
  See [garden.yaml @ 5ef8d0ab16 for more details](https://gitlab.com/garden-rs/garden/-/raw/5ef8d0ab16a64660fef2bfc551e69cc782dfd4a3/garden.yaml).
  Packagers can use `cargo install` to install `garden` and invoke `mdbook` directly
  to install the user manual. We also provide
  `garden -D DESTDIR=/tmp/stage -D prefix=/usr/local install-doc` if distros
  want to install the user manual using our recipe.
  ([#8](https://github.com/davvid/garden/pull/8))

- Garden's command-line parsing has been rewritten to leverage the
  [clap](https://crates.io/crates/clap) crate and ecosystem.

## v0.4.1

*Released 2022-12-26*

**Features**:

- The `garden cmd --no-errexit` option was extended to work with commands that are
  configured using a YAML list of strings. Commands that are specified using lists
  are now indistinguishable from commands specified using multi-line strings.

## v0.4.0

*Released 2022-12-23*

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

- `garden cmd` can now run commands in breadth-first order when the
  `-b/--breadth-first` option is used. Depth-first tree traversal is the default.
  The `garden cmd --breadth-first` traversal runs all commands on a tree before
  continuing on to the next tree. The default `garden cmd` depth-first traversal
  runs a command across all trees before continuing on to the next command.
  ([#3](https://github.com/davvid/garden/issues/3))

## v0.3.0

*Released 2022-08-20*

**Features**:

- `garden plant` can now detect `git worktree` repositories.
  ([#1](https://github.com/davvid/garden/issues/1))

## v0.2.0

*Released 2022-07-29*

**Breaking Changes**:

- `garden add` was renamed to `garden plant`.

**Features**:

- `garden grow` can now grow trees using "git worktree" (#1).
- `garden grow` learned to clone specific branches.
- `garden grow` and `garden plant` can now handle bare repositories.


## v0.1.0

*Released 2022-06-13*

**Features**:

This is the initial garden release.

- `garden grow` grows worktrees.
- `garden init` initializes configuration.
- `garden plant` (formerly `garden add`) adds existing trees.
- `garden cmd` and `garden <custom-command>` can run custom commands.
- Templates, variables, and environment variables are all supported.
