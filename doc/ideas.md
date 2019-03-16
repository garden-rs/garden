# Ideas


## Features


### Symlinks

Symlink trees create a symlink on the filesystem during `garden init`.
`garden exec`, and custom `garden cmd` commands ignore symlink trees.

    trees:
        media:
            path: ~/media
            symlink: /media/${USER}


### Allow globs when specifying group members

    groups:
        example: foo/*


### Allow globs when specifying garden trees and groups

    gardens:
        example:
            groups: dev/*
            trees: beta/*


### garden inspect

    garden inspect <tree-expr>

Show the repository status for the resolved trees.


# vim: set ts=4 sw=4 sts=4
