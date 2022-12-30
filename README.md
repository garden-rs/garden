# Garden

Garden grows and cultivates collections of Git trees.

Garden helps you define development workflows that operate over collections of
self-contained and interdependent Git worktrees.

## Code Status

[![Build status](https://github.com/davvid/garden/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/davvid/garden/actions/workflows/ci.yml)
[![MIT License](https://img.shields.io/github/license/davvid/garden.svg)](LICENSE)

Garden is under development and not yet feature complete, but its current features are
considered stable.

The [ideas](doc/ideas.md) page contains a list of ideas to explore in the future.


## Documentation

Read the [Garden User Guide](https://davvid.github.io/garden)
for details on how to use and configure Garden.

Read the [Garden API Documentation](https://docs.rs/garden-tools/)
for details on how to use the Garden APIs for developing Garden.


## Features

Garden aids in common development setup steps such as setting environment
variables, configuring search paths, and creating arbitrary groupings of
repositories for development.

* Bootstrap Git-based development environments from source.

* Define arbitrary collections of Git repositories for running commands.

* Define environment variables scoped to specific projects or trees.

* Define custom commands and workflows in a simple declarative config file.

* Develop, build and test interdependent projects in self-contained sandboxes.


## Acknowledgements

The structure and content of the README and documentation was heavily inspired
by the the [mdbook](https://github.com/rust-lang/mdBook) documentation.

The [yaml-rust parser used by Garden](https://github.com/davvid/yaml-rust)
is [@davvid](https://github.com/davvid)'s fork of the
[original yaml-rust](https://github.com/chyh1990/yaml-rust) crate by
[@chyh1990](https://github.com/chyh1990).
