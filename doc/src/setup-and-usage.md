# Setup and Usage

## Setup

A garden can be created in any directory. Run `garden init` to create
an empty "garden.yaml" file for defining trees, groups and gardens.

```bash
# Create an empty "garden.yaml" in the current directory.
garden init
```

By default, the garden.yaml is configured to operate on trees in the
current directory relative to the garden file. The `garden` command
searches the current directory for a "garden.yaml" config file.

If no "garden.yaml" could be found in the current directory then `garden`
will attempt to load a "global" configuration file, typically located in
`~/.config/garden/garden.yaml`.

The following `garden init` invocation creates an empty `garden.yaml` with a
root directory of `~/src` in the user's home confiuration directory. This is
typically `~/.config/garden/garden.yaml`.

```bash
garden init --global --root '~/src'
```

This allows you to access the config irrespective of the current directory and
perform garden operations without needing to `cd` into `~/src/` to do so.

If multiple configuration files are made available in `~/.config/garden`, then
using `garden -c alt.yaml ...` from ***any*** directory (without specifying an
absolute path) will use the `alt.yaml` garden file.


## Usage

Garden is used by creating a configuration file and defining a directory
structure for git trees.

Existing trees can be added to the configuration using `garden add`.

The `garden init` command creates an empty `garden.yaml` file from scratch in
the current directory, or in the user-global `~/.config/garden/garden.yaml`
directory if `--global`  is specified.  The global configuration is used when
`garden.yaml` is not found under the current directory.

The `garden grow` command is used to bring trees into existence from an
existing `garden.yaml`.

Garden commands can be used to operate over sets of trees once a configuration
has been defined using `garden add <repo>` or by adding the tree to
`garden.yaml` using your favorite editor.
