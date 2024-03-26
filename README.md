# Garden

Garden grows and cultivates collections of Git trees.

Garden helps you define development workflows that operate over collections of
self-contained and inter-dependent Git worktrees.

## Code Status

[![Build status](https://gitlab.com/garden-rs/garden/badges/main/pipeline.svg)](https://gitlab.com/garden-rs/garden/-/pipelines)
[![MIT License](https://img.shields.io/gitlab/license/garden-rs/garden.svg)](LICENSE)

Garden is actively maintained and its core functionality is stable and feature-complete.


## Documentation

Read the [Garden User Guide](https://garden-rs.gitlab.io)
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

* Leverage your existing shell scripting knowledge. If you already know
(bash/zsh/etc) shell then you can learn to use `garden` with minimal effort.

* Configured using simple YAML files extended with a flexible expression syntax.
Garden helps you define (multi-repository) workflows using the vast ecosystem of
command-line tools.


## Links and Related Projects

* [Garden on crates.io](https://crates.io/crates/garden-tools)

* [Garden Homebrew formula](https://gitlab.com/garden-rs/homebrew-garden)

* [Garden NetBSD pkgsrc](https://cdn.netbsd.org/pub/pkgsrc/current/pkgsrc/devel/garden/index.html)

* [Garden seeds](https://gitlab.com/garden-rs/garden-seeds) ~ reusable templates for garden.


## Acknowledgements

The structure and content of the README and documentation was heavily inspired
by the [mdbook documentation](https://github.com/rust-lang/mdBook).
