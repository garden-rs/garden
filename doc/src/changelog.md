# Changelog

## Upcoming

**Development**:

- `garden-gui` switched over to `egui_autocomplete` now that our
[autocomplete multiple words feature](https://github.com/JakeHandsome/egui_autocomplete/pull/38)
was merged and released.


## v2.2.0

*Released 2025-05-11*

**Features**:

- `garden ls` now has a `-R | --no-remotes` option to disable the display of remotes.

- `garden gui` now avoids displaying duplicate trees.

- `garden --help` now uses ANSI colors.

**Development**:

- The `xdg` dependency was upgraded to v3.0.

- The `idna_adapter` and `mime_guess2` crates were pinned to maintain the
current MSRV of 1.81.0 for `garden-gui`.


## v2.1.0

*Released 2025-02-23*

**Features**:

- Commands defined in Garden and Tree scopes are now runnable from the `garden gui`
command palette.

- `garden grow` is now runnable from the `garden gui` commands and query results.

- `garden ls` is now runnable from the `garden gui` commands and query results.

- Missing trees are now displayed in red in the `garden gui` query results.

- Tree names and paths can now be copied to the system clipboard from the
`garden gui` query results.

- Autocompletion was added to the `garden gui` query field.

- Escape and Ctrl-Q hotkeys can now close the `garden gui` command details window.

- The Ctrl-Q hotkey will now close the main `garden gui` window.

- `garden ls` now has `-G | --no-groups` and `-N | --no-gardens` options
for omitting groups and gardens from its output.

**Development**:

- The `v2.0.0` release removed `Cargo.lock` and added an entry for it to `.gitignore`.
The `.gitignore` entry has now been removed to make it easier for packagers to track
their changes.

- The `yaml-rust2` dependency was upgraded to v0.10.

- The `egui` dependency was upgraded to v0.31.

**Documentation**:

- Windows support was improved in `v1.10.1` to the point where we can now consider
Windows as being officially supported. This development has now been documented
in the installation section of the Garden User Guide.

**Packaging**:

- The Nightly Build artifacts from Gitlab CI have been updated to include `garden-gui`.


## v2.0.0

*Released 2025-01-26*

**Features**:

- A new `garden-gui` graphical user interface for Garden is now available.
Garden GUI is provided as a separate `cargo install garden-gui` crate.

**Development**:

- The `dirs` dependency was upgraded to v6.0.

- The `thiserror` dependency was upgraded to v2.0.

- The `which` dependency was upgraded to v7.0.


## v1.10.1

*Released 2025-01-14*

**Features**:

- `garden completion` now includes a comment in its output indicating which version of
`garden` was used to generate the completion script.

- Windows support was improved by avoiding UNC paths.
([#20](https://gitlab.com/garden-rs/garden/-/issues/20))


## v1.10.0

*Released 2024-12-14*

**Features**:

- `garden grow` can now filter the remotes that will be updated by specifying a glob
pattern to the new `--remote` option.

- `garden ls` now has a `-s | --sort` option that allows you to sort
trees by name or modification time.

**Development**:

- Use of the [unmaintained](https://rustsec.org/advisories/RUSTSEC-2024-0388)
`derivative` crate has been eliminated.


## v1.9.1

*Released 2024-11-06*

**Features**:

- `GARDEN_CMD_VERBOSE` and `GARDEN_CMD_QUIET` are now updated when using
`garden <command> -v ...` and `garden cmd <command> -v ...`.
Previously, these variables were only set when `-v` was used against the `garden`
command directly, before any sub-commands, e.g. `garden -v ...`.

**Fixes**:

- `garden exec`'s parallel mode was made more robust.


## v1.9.0

*Released 2024-10-11*

**Features**:

- `garden exec` can now run commands in parallel using the `-j# | --jobs=#` option.
([#43](https://github.com/garden-rs/garden/issues/43))

**Packaging**:

- Garden's Nix flake was improved and using Garden with Nix home-manager was documented.
([#46](https://github.com/garden-rs/garden/pull/46))
([#17](https://github.com/garden-rs/garden/issues/17))

**Development**:

- Internal APIs for running commands were refactored.

- The `yaml-rust2` dependency was upgraded to v0.9.


## v1.8.0

*Released 2024-09-26*

**Features**:

- `garden cmd` and custom commands now have a `-j# | --jobs=#` option for
[running commands in parallel](https://garden-rs.gitlab.io/commands.html#parallel-execution).
Use `-j0 | --jobs=0` to use all available cores.
([#43](https://github.com/garden-rs/garden/issues/43))
([#45](https://github.com/garden-rs/garden/pull/45))

- `garden ls` now has a `--reverse | -r` option to display trees in reverse order.
([#44](https://github.com/garden-rs/garden/pull/44))

**Development**:

- The `which`, `yansi` and `strum` dependencies were upgraded. `yansi` was a new major version
and required a fair amount of changes. `strum` involved minor changes.
([#42](https://github.com/garden-rs/garden/pull/42))


## v1.7.0

*Released 2024-06-29*

**Features**:

- `garden ls` now has a `--commands | -c` option to display just commands.
The related `--no-commands | -C` option is used to omit commands from being displayed.
([#39](https://github.com/garden-rs/garden/issues/39))
([#41](https://github.com/garden-rs/garden/pull/41))

- `garden cmd` and `garden <custom-command>` now support a `--dry-run | -N` option
to perform trial runs without actually running any commands.
([#39](https://github.com/garden-rs/garden/issues/39))
([#41](https://github.com/garden-rs/garden/pull/41))

- `garden exec` made `-N` the short option for its `--dry-run` option and the original
`-n` short option was made an undocumented alias for compatibility.
([#41](https://github.com/garden-rs/garden/pull/41))

- The `garden eval`, `garden exec`, `garden cmd` and custom sub-commands
now accept the same `--define | -D name=value` override options as the root
`garden` command.

- `garden grow` reports more details about the commands it runs and no
longer prints redundant `git config` commands.

**Fixes**:

- `garden ls` now prints the list of commands in the same order as they appear in `garden.yaml`.
([#39](https://github.com/garden-rs/garden/issues/39))
([#41](https://github.com/garden-rs/garden/pull/41))

**Packaging**:

- The nix flake was updated to re-enable llvm coverage.
([#38](https://github.com/garden-rs/garden/pull/38))

- `nix run` can now be used to run `garden` and `nix shell` can now be used to
open a nix shell with garden installed.
([#40](https://github.com/garden-rs/garden/pull/40))

**Development**:

- More structs, functions and methods were made private.

- Several types were renamed from "HashMap" to "Map".


## v1.6.0

*Released 2024-06-02*

**Features**:

- `zsh` is now invoked using `zsh +o nomatch` for better portability across
shells. This prevents zsh from erroring when wildcard patterns find
no matches. Wildcards can be used, for example, to implement a
custom `clean` command that feeds `rm -f` using wildcard patterns,
but these commands would generate errors without disabling `nomatch`.
The zsh `nomatch` option is a less useful option for non-interactive use
so we disable it unconditionally.

- The `--verbose | -v` option can now be passed to custom and built-in commands.
The `verbose` option was previously a global option that had to
be specified before sub-commands. The following invocations are all
equivalent now:
  - `garden -vv build`
  - `garden -v build -v`
  - `garden build -vv`

  ([#36](https://github.com/garden-rs/garden/pull/36))

**Packaging**:

- The nix flake was updated to use Fenix for the latest stable rustc 1.78.0.
([#37](https://github.com/garden-rs/garden/pull/37))

**Development**:

- An `.envrc` file was added to enable the nix flake for direnv users.
([#37](https://github.com/garden-rs/garden/pull/37))


## v1.5.0

*Released 2024-04-14*

**Features**:

- Running `garden init` inside a Git repository will now record the
current directory as a tree with its path set to  `${GARDEN_CONFIG_DIR}`.
([#34](https://github.com/garden-rs/garden/pull/34))

- Custom commands skip missing trees by default. A new `-f | --force`
option can be used to make `garden` run commands on missing trees.
([#33](https://github.com/garden-rs/garden/issues/33))

- `garden plant` now avoids updating the configuration when a tree is
re-planted and its configuration contains expressions that evaluate
to the same value as currently exist in git.
([#31](https://github.com/garden-rs/garden/issues/31))
([#32](https://github.com/garden-rs/garden/pull/32))

**Packaging**:

- [Prebuilt binaries](https://github.com/garden-rs/garden/releases)
are now available!

**Development**:

- The original github repository under `davvid`'s namespace was transferred to the
[garden-rs](https://github.com/garden-rs/garden) organization on github.

- The `yaml-rust2` dependency was upgraded to `0.8.0` to avoid the `encoding` crate
([RUSTSEC-2021-0153](https://rustsec.org/advisories/RUSTSEC-2021-0153)).


## v1.4.1

*Released 2024-03-22*

**Features**:

- The empty directory detection in `garden grow` was improved.

**Development**:

- The internal APIs were updated to use `AsRef<Path>` wherever possible.


## v1.4.0

*Released 2024-03-21*

**Features**:

- Custom commands can now specify an interpreter to use on a per-command basis.
If a command uses a shebang `#!` line then the command's text will be passed as the
next argument to the specified command. For example, using `#!python3 -c` as the
first line in a custom command will cause `python3 -c <command>` to be executed.

- Trees can now use branches defined in separate remotes when configuring the
default branch to checkout. `garden grow` will now fetch the remote associated with the
configured branch switching branches in order to make this possible.

- Trees can now use any upstream branch from any configured remote in the `branches` section.
Previously, branches associated with non-default remotes could not be created unless
they were fetched beforehand. `garden grow` will now fetch the associated remote
before creating the local branch.

- `garden grow` now detects empty directories (e.g. the directories that are created
when using uninitialized Git submodules) and will properly clone into the empty directories
instead of treating them like an already-grown tree.
([#30](https://github.com/garden-rs/garden/pull/30))

**Development**:

- `garden` can now be built on Windows. Symlink trees and the XDG base directory support
is UNIX-only and disabled on Windows.
([#17](https://gitlab.com/garden-rs/garden/-/issues/17))

- [yaml-rust2](https://crates.io/crates/yaml-rust2) is now used instead of
the [yaml-rust-davvid](https://crates.io/crates/yaml-rust-davvid) fork that was
being maintained by [@davvid](https://github.com/davvid) for use by garden.
([#29](https://github.com/garden-rs/garden/pull/29))


## v1.3.0

*Released 2024-02-19*

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
([#23](https://github.com/garden-rs/garden/pull/23))

- Evaluation cycles (i.e. circular variable dependencies) are now prevented when
evaluating garden variables. The evaluation engine will now return empty strings
when a variable with a cyclical expression is evaluated.
([#24](https://github.com/garden-rs/garden/pull/24))

- When `zsh` is used as the `garden.shell`, which happens automatically when `zsh`
is installed, `garden` will now use `zsh -o shwordsplit` in order to enable
word-splitting of `$variable` expressions by default. This makes `zsh` behave
just like other shells by default, which improves the portability of commands.
Configure `garden.shell-wordsplit` to `false` or use the
`garden <cmd> -z | --no-wordsplit` option to opt-out of this behavior.
([#25](https://github.com/garden-rs/garden/pull/25))

- `garden.shell` can now be configured to use arbitrary commands for executing
command strings. Garden uses the configured `garden.shell` as-is and does
not augment its options (e.g. `-e`  or `-o shwordsplit`) when a custom command
is used. Custom commands are identified as commands that expand to 2 or more
command-line arguments. Thus, `python3` is not considered a custom command
and `garden` will run `python3 -c <string>` to run commands. On the other
hand, specifying `ruby -e` *is* considered a custom command because it
expands to `["ruby", "-e"]` under the hood. If you need to use a custom
command  that takes no additional command-line arguments then you can
use `env` as an extra argument to have it be considered as a custom shell.
For example, `env custom-shell` will cause `garden` to run
`env custom-shell <string>`, which is equivalent to `custom-shell <string>`.
Using just `custom-shell` would have resulted in `garden` running
`custom-shell -c <string>` instead, which may not be desired.
([#26](https://github.com/garden-rs/garden/pull/26))

- The `garden shell` command can now be configured to use an interactive command shell
that is distinct from the command specified in the `garden.shell` configuration by
configuring the `garden.interactive-shell` value.
([#26](https://github.com/garden-rs/garden/pull/26))

- `garden shell` can now be run without any arguments. The tree query now defaults to
`.` so that the tree in the current directory is used when nothing is specified.
([#26](https://github.com/garden-rs/garden/pull/26))

- Custom commands now have access to a `${GARDEN_CMD_VERBOSE}` and `${GARDEN_CMD_QUIET}`
variables which can be used to forward the `--verbose` and `--quiet` arguments
down into child `garden` invocations. `${GARDEN_CMD_VERBOSE}` uses the short `-v`
flag in the value to support the case where the verbose option is specified
multiples times to increase the verbosity level (e.g. `-vv`).
([#27](https://github.com/garden-rs/garden/pull/27))


## v1.2.1

*Released 2024-02-05*

**Development**:

- The `yaml-rust-davvid` dependency was upgraded to `v0.6.0`.

- Documentation and code maintenance.


## v1.2.0

*Released 2024-01-27*

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

*Released 2024-01-15*

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
([#3](https://github.com/garden-rs/garden/issues/3))
([documentation](https://garden-rs.gitlab.io/commands.html#pre-and-post-commands))

- The default `origin` remote name can now be configured using `tree.<tree>.default-remote`.
([#16](https://gitlab.com/garden-rs/garden/-/issues/16))

- Commands now display the tree's current branch alongside the tree name.
([#18](https://github.com/garden-rs/garden/issues/18))

- `garden -vv exec` and `garden -vv shell` now display the command being run.

**Packaging**:

- `garden` can now be installed as a `nix flake` package.
A `flake.nix` file is now provided.
([#16](https://github.com/garden-rs/garden/issues/16))

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
([#14](https://github.com/garden-rs/garden/issues/14))
([#15](https://github.com/garden-rs/garden/pull/15))

- [Trees now sparsely override existing entries](https://garden-rs.gitlab.io/configuration.html#l#the-last-one-wins-rule).
This behavior allows a tree definition to replace just the `url` field, or to replace
individual tree commands while retaining the rest. Use `replace: true` in a Tree
definition in order to completely replace the existing entry instead of sparsely
overriding it.

- Improved shell completions for `garden`, `garden init` and `garden plant`.

**Packaging**:

- [0323pin](https://github.com/0323) packaged `garden` for pkgsrc/NetBSD and
[merged the package into the main branch!](http://mail-index.netbsd.org/pkgsrc-changes/2023/01/22/msg267560.html)
([#13](https://github.com/garden-rs/garden/issues/13))

## v0.6.0

*Released 2023-01-20*

**Features**:

- Both names and values in `gitconfig` can now use `${var}` expressions.
Previously only values were evaluated. Names are evaluated now as well.

**Fixes**:

- The `zsh` workaround for `garden completion zsh` is no longer needed.
The [documentation for generating zsh completions](https://garden-rs.gitlab.io/commands.html#zsh)
has been updated.
([#10](https://github.com/garden-rs/garden/issues/10))

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
([#7](https://github.com/garden-rs/garden/pull/7))

- [Garden commands can now use shell variables](https://garden-rs.gitlab.io/commands.html#shell-syntax)
using the standard (brace-less) shell `$variable` syntax. The braced `${garden}`
variable syntax remains reserved for resolving Garden Variables.
Double-`$` braces (ex: `$${...}`) can be used to escape a `$${variable}` from
evaluation so that a literal `${variable}` value is used in the garden command.
([#11](https://github.com/garden-rs/garden/issues/11))
([#12](https://github.com/garden-rs/garden/pull/12))

- A new `garden completion` subcommand was added for providing shell command-line
completion using the [clap_complete](https://crates.io/crates/clap_complete) crate.
([#9](https://github.com/garden-rs/garden/pull/9))

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
([#8](https://github.com/garden-rs/garden/pull/8))

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
([#4](https://github.com/garden-rs/garden/issues/4))

- `garden cmd` can now run commands in breadth-first order when the
`-b/--breadth-first` option is used. Depth-first tree traversal is the default.
The `garden cmd --breadth-first` traversal runs all commands on a tree before
continuing on to the next tree. The default `garden cmd` depth-first traversal
runs a command across all trees before continuing on to the next command.
([#3](https://github.com/garden-rs/garden/issues/3))

## v0.3.0

*Released 2022-08-20*

**Features**:

- `garden plant` can now detect `git worktree` repositories.
([#1](https://github.com/garden-rs/garden/issues/1))

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
