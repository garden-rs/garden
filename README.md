# Garden

Garden grows and cultivates collections of Git trees.

Garden is a simple yet expressive command runner and multi-repo Git configuration tool.
Garden helps you create development workflows over collections of self-contained,
loosely-coupled and inter-dependent Git worktrees.

```bash
# Install the "garden" command-line tool.
cargo install garden-tools

# Install the "garden-gui" graphical user interface for Garden.
cargo install garden-gui
```

## Documentation

Read the [Garden User Guide](https://garden-rs.gitlab.io)
for details on how to use and configure Garden.

Read the [Garden API Documentation](https://docs.rs/garden-tools/)
for details on how to use the Garden APIs for developing Garden.


## Installation

* [Garden installation guide](https://garden-rs.gitlab.io/installation.html)

* [Garden pre-built binaries](https://github.com/garden-rs/garden/releases)

Pre-built `garden` binaries are statically linked using
[musl libc](https://musl.libc.org/) so that they can run on any Linux system.

Pre-built binaries for Garden's graphical user interface (`garden-gui`) are only
available as amd64/x86_64 Linux tarballs and Debian packages. `garden-gui` is provided
by the `garden-gui_*_amd64.deb` and `garden-gui-*-x86_64-unknown-linux-gnu.tar.gz`
artifacts.

Both `garden` and `garden-gui` must be installed in order to use `garden-gui`.


## Use Cases

* Garden bootstraps Git-based multi-repo development environments from source.
Garden can store and apply `git config` and `git remote` configuration to existing
or new Git worktrees that Garden can "grow" (clone) into existence.

* Garden runs commands over collections of Git repositories.
The simplicity of Garden's syntax and its dynamic expression variables
makes it a viable replacement for `make` when used as a simple task runner.

* Garden is configured using YAML files alongside a familiar UNIX shell syntax that
leverages your existing shell knowledge. If you already know POSIX/bash/zsh shell then
you can learn to use `garden` with minimal effort.


## Links and Related Projects

* [Garden on crates.io](https://crates.io/crates/garden-tools)

* [Garden GUI on crates.io](https://crates.io/crates/garden-gui)

* [Garden Homebrew formula](https://gitlab.com/garden-rs/homebrew-garden)

* [Garden NetBSD pkgsrc](https://cdn.netbsd.org/pub/pkgsrc/current/pkgsrc/devel/garden/index.html)

* [Garden seeds](https://gitlab.com/garden-rs/garden-seeds) ~ reusable templates for garden.


## Code Status

[![Build status](https://gitlab.com/garden-rs/garden/badges/main/pipeline.svg)](https://gitlab.com/garden-rs/garden/-/pipelines)
[![MIT License](https://img.shields.io/gitlab/license/garden-rs/garden.svg)](LICENSE)

Garden is actively maintained and its core functionality is stable and feature-complete.
