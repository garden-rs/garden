# Ideas


## Fixups

- Empty environment varibles should not get a ":"


## Features


### garden shell

shlex.split() the shell expression.


### garden init

    garden init <tree-expr>

Initialize missing repositories for the resolved trees.


### Add group to TreeContext

Add an Option(GroupIndex) to TreeContext so that environment()
expansion can be extended to work on groups as well as gardens.


### Tree symlinks

Support "symlink" trees whose purpose is to define the existence
of a symlink on the filesystem only.

Example:

    trees:
        media:
            symlink: ~/media
            path: /media/${USER}


### Allow globs when specifying group members

    groups:
        example: foo/*


### Allow globs when specifying garden trees and groups

    gardens:
        example:
            groups: dev/*
            trees: beta/*


###  Allow ${variables} in environment variables names

    environment:
        RP_${TREE_NAME}=: ${prefix}


### garden inspect

    garden inspect <tree-expr>

Show the repository status for the resolved trees.


# vim: set ts=4 sw=4 sts=4
