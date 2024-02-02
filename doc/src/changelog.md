# Changelog

## v1.3.0

**Features**:

- `garden eval` and garden expressions in general will now resolve variables defined
  within `environment` blocks. Environment blocks were previously not considered when
  resolving variables. Environment blocks are now resolved and checked for variables
  when `${variable}` expressions do not find the variable in scopes with higher
  precedence. The precedence order, from strongest to weakest, is the `variables`
  block in a garden's scope, the `variables` block in a tree's scope, the
  `variables` block in global configuration scope, the `environments` block in
  a garden's scope, the `environments` block in a tree's scope, the
  `environments` block in global configuration scope and, lastly, OS environment
  variables. The first entry found is used when expanding variable expressions.


## v1.2.1

*Released 2023-02-05*

**Development**:

- The `yaml-rust-davvid` dependency was upgraded to `v0.6.0`.

- Documentation and code maintenance.


## v1.2.0

*Released 2023-01-27*

**Features**:

- If a garden file has no trees defined at all then an implicit tree called
  `.` will now be synthesized into existence. The presence of this implicit tree
  allows garden files that define just the current directory as a tree to omit
  the entire `trees` section altogether. This consequently makes it easier
  to use garden as a simple command runner because the `commands` section
  is the only section required in order to run `garden` commands.

- When `garden.root` is not configured `garden` will behave as if
  `garden.root` is configured to `${GARDEN_CONFIG_DIR}`. This allows garden files to
  omit `garden.root` from their configuration in typical scenarios.

- Configuring `garden.root` to an empty string (`""`) will behave as if `garden.root`
  is configured to the current directory from which `garden` was run.

- When a `garden.yaml` file does not exist in the current directory then garden
  will walk up the file system searching for `garden.yaml` or the name specified using
  `garden -c <name>`. Garden will behave as if it were launched from the directory
  containing the garden file when a configuration file is found.

- The `GARDEN_CEILING_DIRS` and `GIT_CEILING_DIRS` environment variables can be
  used to limit the `garden.yaml` discovery by preventing `garden` from traversing
  into and beyond the specified directories when discovering garden files.

- `garden exec`, `garden cmd` `garden grow`, `garden ls` and custom garden commands
  can now filter the trees they operate over by passing a glob pattern using
  `-t | --trees` option. These commands will only operate on the trees whose names
  match the pattern. This allows you to specify a garden as the tree query and use
  the full set of environment variables from all trees in the query while
  executing commands over a subset of the trees in that garden.

- `garden init` will now add the current directory to the `trees` block
  when the current directory contains a Git repository. Use `garden init --empty`
  to disable this behavior.

**Development**:

- The `shlex` dependency was upgraded to `1.3.0`, which includes fixes for
  [RUSTSEC-2024-0006](https://rustsec.org/advisories/RUSTSEC-2024-0006.html) a.k.a.
  [GHSA-r7qv-8r2h-pg27](https://github.com/comex/rust-shlex/security/advisories/GHSA-r7qv-8r2h-pg27).

## v1.1.0

*Released 2023-01-15*

**Features**:

- `garden ls` now displays information about trees, groups, gardens and commands.

- `garden ls -c` (i.e. `--no-commands`) hides command details from the output.

- `garden plant -s` (i.e. `--sort`) sorts all of the configured `trees` entries
  after planting the specified trees.

- `garden exec -n` (i.e. `--dry-run`) performs a trial run without executing
  any commands.

- `garden.shell-errexit` can now be configured to `false` in `garden.yaml` to
  opt-out of using the `-e` exit-on-error shell option when running custom commands.

- `garden.shell` can now be configured to `bun`, `fish`, `node`, `perl` and `python3`
  in addition to the traditional `bash`, `zsh`, `dash`, `ksh` and `sh` shells.
  This allows you to use these interpreters to run custom commands.

**Development**:

- Garden is now using
  [shellexpand v3](https://gitlab.com/ijackson/rust-shellexpand#version-300-2022-12-01).

## v1.0.0

*Released 2023-12-23*

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

*Released 2023-11-19*

**Fixes**:

- `garden -D name=value` now overrides variables in all scopes.
  Variables defined in tree scope were not subject to overrides and
  will now get properly overridden by the `--define` / `-D`
  command-line options.

## v0.9.0

*Released 2023-11-02*

**Features**:

- `garden grow` now sets `git config remote.$name.tagopt --no-tags`
  when adding additional remotes. This prevents accidentally fetching tags
  when interacting with remotes.

## v0.8.1

*Released 2023-07-18*

**Fixes**:

- `garden grow` was fixed to detect existing branches when growing
  `git worktree`-created child worktrees.

**Development**:

- `strum` is now used to implement `FromStr` for `enum ColorMode`.

- [is-terminal](https://crates.io/crates/is-terminal) is now used instead of
  the unmaintained `atty` crate.

## v0.8.0

*Released 2023-07-16*

**Features**:

- `garden` now supports a `grafts` feature that allows you to
[stitch configuration entities from separate garden files](https://garden-rs.gitlab.io/configuration.html#grafts)
into a graft-specific namespace. Trees and variables from grafted configurations can be
referenced using `graft::` namespace qualifiers.

- `garden grow` can now configure [upstream branches](https://garden-rs.gitlab.io/commands.html#upstream-branches).

- `garden grow` can now configure [gitconfig settings with multiple values](https://garden-rs.gitlab.io/commands.html#upstream-branches#git-configuration-values)
  using [`git config --add <name> <value>`](https://git-scm.com/docs/git-config#Documentation/git-config.txt---add).

## v0.7.0

*Released 2023-02-12*

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

- [Garden commands can now use shell variables](https://garden-rs.gitlab.io/commands.html#shell-syntax)
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
