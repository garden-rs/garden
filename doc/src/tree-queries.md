# Tree Queries

Garden commands use "Tree Query" arguments that specify which groups, gardens or trees
to operate on. Tree Queries are strings that resolve to a set of trees.

Strings with garden, group or tree names, `@tree` references, `%group` references,
`:garden` references and wildcards are all Tree Queries.

When a query is specified, garden will use the first matching garden, group or
tree, in that order, when determining which trees to operate on.  When a
garden or group is matched, all of its associated trees are used.


## Tree Queries Resolve to Multiple Trees

The "cola" garden in the configuration section contains multiple trees that can be
operated on together by specifying "cola" as the tree query.

```bash
# Run "git status -s" over each tree in the "cola" garden.
garden exec cola git status -s
# Run the custom "status" command over each tree in the "cola" garden.
garden status cola
# Run the "status" and "build" commands over each tree in the "cola" garden.
garden cmd cola status build
```


## Wildcards

Garden understands shell wildcards in tree queries. Glob wildcards allow commands to
operate on multiple sets of gardens, groups or trees.

```bash
# Clone trees whose name starts with "git" and run "pwd" in each tree.
garden grow '@git*'
garden exec '@git*' pwd
```


## Paths

Paths can be used as a tree query as long as the specified directory refers to a tree
known to Garden. If the path does not correspond to a tree known to garden then no
commands will be run.

```bash
# Run "build" on the tree in the current directory and pass "--verbose" to the command.
garden build . -- --verbose
```

The file system has the lowest priority relative to gardens, groups, and trees when
expanding a tree query. If an argument is ambiguous and can, for example, refer to both
a "group" name and a single tree's path on the file system, garden will interpret the
argument as a "group".

Please note that `garden` will use `.` (dot, the current directory in file system jargon)
as the default tree query for custom commands when no tree query is specified.

```bash
# This...
garden build
# is equivalent to specifying "." as the tree query.
garden build .
```


## Resolving Trees, Groups or Gardens Only

If you have groups, gardens, and trees with the same name then you can use the
`@tree`, `%group`, and `:garden` prefixes to disambiguate the query.

* ***@tree*** - values prefixed with `@` resolve trees only
* ***%group*** - values prefixed with `%` resolve groups only
* ***:garden*** - values prefixed with `:` resolve gardens only

```bash
garden grow @tree      # grow the tree called "tree"
garden grow %group     # grow the group called "group"
garden grow :garden    # grow the garden called "garden"
```

When no prefixes are used then the names are resolved in a specific order.
Gardens have the highest priority, followed by groups, trees and lastly paths.

If your trees, groups and gardens are named uniquely then you will rarely need to
use prefixes in your tree queries.
