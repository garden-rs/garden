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


### Custom sub-commands

    garden <command>  <tree-expression>*
    garden diff @cola

Extend the argument parser to allow for custom commands to be used when
no builtin garden command exists by that name.  This should resolve
the tree context first so that the command is evaluated  in the
tree context.

This is complementary to `garden cmd <tree-expression> <command>*`
because that allows multiple commands; this form allows multiple
tree-expressions, and is convenient to type.


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


### garden status

    garden status <tree-expr>

Show the repository status for the resolved trees.


### garden -c foo ...

    garden -c foo ...
    garden -c foo.yaml ...  # both stil use the search path

Search for foo.yaml instead of garden.yaml.
This allows for easy swapping of different configurations.

# vim: set ts=4 sw=4 sts=4
