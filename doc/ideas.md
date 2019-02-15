# Ideas

## Commands

### garden shell

    garden shell <tree> [<garden>]

Launch a shell inside the environment formed by the tree context.
An optional garden can be specified to provide a garden context.


### garden init

    garden init <tree-expr>

Initialize missing repositories for the resolved trees.


### garden add

    garden add <path>

Add an existing tree to garden.yaml.


### garden eval

    garden eval <string> <tree> [<garden>]

Evaluate a string in the specified tree context.


### garden status

    garden status <tree-expr>

Show the repository status for the resolved trees.


## Features

### Tree symlinks

Support "symlink" trees whose purpose is to define the existence
of a symlink on the filesystem only.

Example:

    trees:
        media:
            symlink: ~/media
            path: /media/${USER}
