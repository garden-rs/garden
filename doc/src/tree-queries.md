# Tree Queries

Garden commands take arguments that specify which groups, gardens or trees to
operate on. Tree queries are strings that resolve to a set of trees.

Command arguments with garden, group or tree names, `@tree` references,
`%group` references, `:garden` references and wildcards are all referred to as
"tree queries".

When a name is specified, garden will use the first matching garden, group or
tree, in that order, when determining which trees to operate on.  When a
garden or group is matched, all of its associated trees are used.


## Resolving Trees, Groups or Gardens Only

If you have groups, gardens, and trees with the same name then you can use the
`@tree`, `%group`, and `:garden` syntax to disambiguate between them.

* ***@tree*** - values prefixed with `@` resolve trees only
* ***%group*** - values prefixed with `%` resolve groups only
* ***:garden*** - values prefixed with `:` resolve gardens only

```bash
garden grow @tree      # grow the tree called "tree"
garden grow %group     # grow the group called "group"
garden grow :garden    # grow the garden called "garden"
```

When no prefixes are specified then the names are resolved in the following
order: gardens, groups and trees.

Gardens have the highest priority, followed by groups and lastly trees. If
your trees, groups and gardens are named uniquely then no prefixes are needed.


## Tree Queries Resolve to Multiple Trees

In the following example, the "cola" garden is found in the example
configuration. Each command is run over every tree in that garden.

```bash
# Run "git status -s" over each tree in the "cola" garden.
garden exec cola git status -s
# Run the custom "status" command over each tree in the "cola" garden.
garden status cola
# Run the "status" and "build" commands over each tree in the "cola" garden.
garden cmd cola status build
```


## Paths

Paths can be specified as well, but the filesystem has the lowest priority
relative to gardens, groups, and trees.  When specifying paths they must
resolve to a configured tree.  For example:

```bash
garden build . -- --verbose
```

This runs the `build` command on the tree in the current directory and passes the
`--verbose` flag to the configured `build` command.


## Wildcards

Garden understands shell wildcards.  Glob wildcards in tree queries allows
operations to span over ad-hoc gardens, groups and trees.

This following examples show how wildcards might be used:

```bash
# Grow all all trees whose names start with "git" by cloning them.
garden grow '@git*'

# Run "pwd" in all of the same trees.
garden exec '@git*' pwd
```
