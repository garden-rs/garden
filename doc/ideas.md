# Ideas, New Features, Usability Enhancements

## Set gitconfig values in garden init

    trees:
        example:
            gitconfig:
                remote.origin.annex-ignore: true

## Allow globs when specifying group members

    groups:
        example: foo/*


## Allow globs when specifying garden trees and groups

    gardens:
        example:
            groups: dev/*
            trees: beta/*


## garden inspect

    garden inspect <tree-expr>

Show the repository status for the resolved trees.
