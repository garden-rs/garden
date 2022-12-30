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
root directory of `~/src` in the user's home configuration directory. This is
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

Now that we know how to create a `garden.yaml` we can start using garden to
track our development repositories.

When we have a `~/src/garden.yaml` file with Git worktrees alongside it in the
`~/src` directory then we can start adding those trees to the garden file.
Existing trees are added to a garden file using `garden plant <tree>`.

Once we have a garden file with trees in it then we can use the garden file to
recreate our setup using `garden grow`. The `garden grow` command brings trees
into existence by cloning trees that have been configured in a garden file.

Garden commands are used to operate over sets of trees once a configuration
has been defined. See the [Garden Commands Documentation](commands.md) for
detailed information about the built-in garden commands.
