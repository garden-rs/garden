# Garden User Guide

## Introduction

Garden streamlines development workflows that involve a loosely-coupled set of
multiple, independent Git trees.

Garden allows you to define dynamic relationships and workflows between these
repositories using a YAML configuration file that can be shared and used as
a bootstrapping mechanism for getting an audit-able, from-source project cloned,
built, installed and running with minimal effort for consumers of a Garden file.

Garden sits above any individual project's build scripts and conventions.
Garden is all about making it easy to remix and reuse libraries maintained
in separate Git repositories.

### Project Links

* [Garden source code repository](https://gitlab.com/garden-rs/garden)

* [Garden crates.io package](https://crates.io/crates/garden-tools)

* [Garden Homebrew formula](https://gitlab.com/garden-rs/homebrew-garden)

* [Garden NetBSD pkgsrc](https://cdn.netbsd.org/pub/pkgsrc/current/pkgsrc/devel/garden/index.html)

* [Garden API Documentation](https://docs.rs/garden-tools)


## How Can Garden Help Me?

Garden's features are useful if you find yourself in any of these roles.

### The Tinkerer

If you find yourself exploring lots of projects and building interesting code from
source then Garden can help you keep track of your experiments.

You might have a few dozen Git trees (repositories) cloned and each of them has their
own unique quirks and workflows that you want to be able to jot down and come back to
at any point in the future.

You can create a Garden `garden.yaml` config to keep track of which trees you've
cloned and where they exist on disk. You can remind yourself about the tree by
filling in the `description` field for the tree in your config to remind yourself
in the future.

You can easily recreate the same set of trees on other machines by copying your config
to any other machine and then running a single `garden grow '*'` command to grow all of
your trees from scratch. One way to share your config across multiple machines is to
keep your config in a Git repository so that you can also keep track of changes to
your config.

Each cloned tree can have unique build and workflow recipes that you want to run when
interacting with it. You can write custom commands alongside the tree definitions in your
config to keep track of these command recipes. This lets you, for example, jump into any
trees and run a command such as `garden build` to run the custom `build` command that
you've defined for that tree.

You might have dozens of trees and don't want to have to `cd` around to interact
with each one. Garden lets you invoke commands over trees without needing to
jump around. Garden will internally `chdir` into each tree before running
commands so that you can stay put while running commands in the context
of each tree.

Garden lets you collect trees into groups so that you can run the same-named custom
command across multiple trees using a single command. For example, running
`garden build your-group` will run the `build` command across every tree in
`your-group`. Likewise, `garden grow 'abba*'` will grow just the trees whose names match
the `abba*` shell glob pattern.

Custom commands can be defined in a tree's scope so that a command such as
`garden build` has a different definition in the context of each tree.
Custom commands that do not need to vary per-tree can also be defined at a global
scope to make the command available to every tree.

### The Project Maintainer

You might want to keep track of the forks to your project so that you
can interact with them using `git fetch <remote>`. You might also have
some custom `git config` settings that you want to apply to all of your
worktrees. Garden lets you store your Git remotes and custom Git config
settings in a config file so that you can recreate these settings at
any time using `garden grow .` to apply Garden's settings to the tree
in your current directory.

### The Developer

You find yourself wanting to define custom commands for your project and
want a command runner that is simpler than `make`. You want these same
command recipes to run under CI, so you want to define them in a single
place that is not the forge-specific CI configuration file.
Garden is a simple command runner that is well-suited for this use case,
as its capabilities easily scale down for use within a single tree.
