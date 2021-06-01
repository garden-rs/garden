# Garden

Garden grows and cultivates collections of Git trees.

Garden makes it easy to perform development tasks over collections of
self-contained and inter-dependent Git worktrees.

## Code Status

[![Build Status](https://travis-ci.com/davvid/garden.svg?branch=main)](https://travis-ci.com/davvid/garden)
[![MIT License](https://img.shields.io/github/license/davvid/garden.svg)](LICENSE)

While Garden is under heavy development and not yet feature complete, it is
quite stable and works well for day to day use.

The [ideas](doc/ideas.md) page contains a list of ideas to explore in the future.


## Documentation

Read the [Garden User Guide](https://davvid.github.io/garden)
for details on how to use and configure Garden.

Read the [Garden API Documentation](https://docs.rs/garden-tools/)
for details on how to use the Garden APIs for developing Garden.


## Features

* Bootstrap complex Git-based development environments from source.

* Define arbitrary collections of Git repositories for performing operations.

* Define environment variables scoped to specific projects or trees.

* Define custom commands and workflows in a simple declarative config file.

* Develop, build and test inter-dependent projects in self-contained sandboxes.

Garden weaves together arbitrarily complex development environments from
independent Git worktrees.

Garden aids in common development setup steps such as setting environment
variables, configuring search paths, and creating arbitrary groupings of
repositories for development.


## Installation

There are multiple ways to install garden.

* **From Crates.io**

  This requires at least Rust 1.45 and Cargo to be installed. Once you have
  installed Rust, type the following in the terminal:

  ```
  cargo install garden-tools
  ```

  This will download and compile garden for you. The only thing left to do is
  to add the Cargo `bin/` directory (typically `$HOME/.cargo/bin`) to `$PATH`.

* **From Git**

  The version published to crates.io will often be slightly behind the source
  code repository. If you need the latest version then you can build
  the Git version of Garden yourself. Cargo makes this super easy!

  ```
  cargo install --git https://github.com/davvid/garden.git garden-tools
  ```

  Again, make sure to add the Cargo `bin/` directory to your `$PATH`.

* **For Development**

  If you would like to develop features and contribute to Garden then you will
  have to clone the repository on your local machine.

  ```
  git clone git://github.com/davvid/garden.git
  cd garden
  cargo build
  ```

  The resulting `garden` binary will be located in `target/debug/`.


## Usage

Garden is primarily used as a command-line tool, even though it exposes all of
its functionality as a Rust crate.

Here are the main commands you will want to run. For a more exhaustive
explanation, check out the [User Guide].

- `garden init [<filename>]`

    The init command will create an empty Garden YAML file with the minimal
    boilerplate to start using garden. If the `<filename>` parameter is
    omitted, "garden.yaml" in the current directory will be used.

    This is typically run without specifying a filename, eg. `garden init`.

    ```
    current-directory/
    └── garden.yaml
    ```

- `garden grow {tree,group,garden}`

    If you have a "garden.yaml" file, either one that you authored yourself or
    one that was provided to you, then you will need to grow the Git trees
    into existence. The grow command clones either individual trees, or
    collections of trees when a group or garden name is specified.

- `garden add <worktree>`

    Add an existing Git worktree to an existing "garden.yaml".
    The "trees" section in the configuration file will be updated with details
    about the new tree.

- `garden exec <tree-query> <command-arguments>...`

    Run arbitrary commands over the queried trees, groups or gardens.
    When the `<tree-query>` resolves to a garden then the environment
    is configured for the command using the garden's context.

    The specified command is run on each tree in the resolved query.

- `garden shell <tree-query> [<tree>]`

    Configure the environment and launch a shell inside a garden environment.
    If `<tree>` is specified then the current directory will be set to the
    tree's directory.

- `garden cmd <tree-query> <command> [<command>]...`

    Run one or more user-defined commands over the gardens, groups or trees
    that match the specified tree query.

    For example, if you have a group called "my-group" and two custom commands
    called "build" and "test", then running `garden cmd my-group build test`
    will run the custom "build" and "test" commands over all of the trees in
    "my-group".

- `garden <command> <tree-query> [<tree-query>]*`

    This form is another way to execute user-defined `<command>`s. This form
    allows you to specify multiple queries rather than multiple commands.

    Using the same example as above, `garden build my-group my-other-group`
    will run our user-defined "build" command over both "my-group" and
    "my-other-group".

- `garden eval 'string-with-$vars'`

    Evaluate the specified string using Garden's variable substitution logic.
    The resulting value is printed to stdout.

- `garden help <command>`

    Show the usage help screen for `<command>`. This only works for built-in
    commands and is equivalent to `garden <command> --help`.


## Acknowledgements

The structure and content of the README and documentation was heavily inspired
by the the [mdbook](https://github.com/rust-lang/mdBook) documentation.

The [yaml-rust parser used by Garden](https://github.com/davvid/yaml-rust)
is [@davvid](https://github.com/davvid)'s fork of the
[original yaml-rust](https://github.com/chyh1990/yaml-rust) crate by
[@chyh1990](https://github.com/chyh1990).
