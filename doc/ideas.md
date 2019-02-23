# Ideas

## Features

### Tree symlinks

Support "symlink" trees whose purpose is to define the existence
of a symlink on the filesystem only.

Example:

    trees:
        media:
            symlink: ~/media
            path: /media/${USER}


###  Allow ${variables} in environment variables names

    environment:
        RP_${TREE_NAME}=: ${prefix}


## Commands


### garden init

    garden init <tree-expr>

Initialize missing repositories for the resolved trees.


### garden shell

    garden shell <tree> [<garden>]

Launch a shell inside the environment formed by the tree context.
An optional garden can be specified to provide a garden context.


### garden add

    garden add <path>

Add an existing tree to garden.yaml.


### garden inspect

    garden inspect <tree-expr>

Show the repository status for the resolved trees.


# vim: set ts=4 sw=4 sts=4
