# Ideas, New Features, Usability Enhancements

- grow: make --depth optional and configurable.
- grow: add "grow: branch: clone-branch" and let it be overridden per-graden
  using "gardens: <garden>: branch: <tree>: branch-name"

- config::reader should stop creating `${TREE_NAME}`, `${GARDEN_ROOT}`,
  and `${GARDEN_CONFIG_DIR}`.  config.rs should create the NamedVariables.

- Modular/reusable garden.yaml configuration.

- Make it possible to full-on include another garden config file inline
  into a garden file so that we can reuse variables and commands across
  garden files.

    garden:
        root: "${GARDEN_CONFIG_DIR}"
        include:
          - "${GARDEN_CONFIG_DIR}/etc/garden/commands.yaml"
          - "${GARDEN_CONFIG_DIR}/etc/garden/variables.yaml"

This reader will have to be changed to first parse the includes in the
listed order copy their contents into the main configuration.
This seems like this should be possible to attain through refactoring that
allows the configuration to be iteratively updated by a config reader.
Allow "graft" gardens by including the garden.yaml from an external garden.yaml
and placings its tree, groups, and gardens in a "custom::" graft namespace.

## Grafts (WIP)

Make it possible to graft configuration from one graden file into another.
Trees, variables and groups are available under a "<graft>::" namespace
that can be evaluated from the scope of the main garden file.
Grafts are nestable, as a config inside a namespace can itself have grafts.

    grafts:
        libs: libs/garden.yaml
        internal:
            path: libs/internal.yaml
            root: libs/internal

    trees:
        server: ${vcs}/server
        client: ${vcs}/client

    groups:
        app: [libs::http, libs::rest, internal::auth, server, client]


The trees, gardens, and groups from libs/garden.yaml can be accessed via
`libs::*`. Its root is "libs/" by default.

In order to resolve variables, we have to start at the leaf-most
Configuration and walk up the parent Configurations until the variable is
resolved.

But, when a variable reference contains "ns::" namespace/graft prefixes
then we need to walk down the hierarchy, starting from the current
Configuration down to child grafts.

The configuration reader strategy is to first read the top-level
configuration.  When a configuration is read, stub Graft entries are read and
recorded in Configuration::grafts so that they can be read later.

Once the Configuration has been read, child Configuration grafts are read and
stitched into the parent graft.  At this point the parent's NodeId is recorded
into the child Configuration so that traversal can use this information to
find the parent Configuration when resolving variables.

To support top-down traversal, the child graft Configuration NodeId is
recorded into the parent Configuration's graft entry.


- Stategy for Tree contexts

When an Option<NodeId> is present in the tree context then the value must
be evaluated using the Configuration corresponding to the NodeId.
The NodeId looks up the Configurtion for that context.


- Strategy for evaluating values

`query::tree_context(query: String)` resolves a string to a TreeContext that
represents a Tree in a particular garden or grafted configuration.

When the query contains "the-graft::tree" identifiers then we first attempt to
find a graft that matches the name.  If the graft with by the name of
"the-graft" exists then the child configuration is looked up for "the-graft"
and the query is resolved in the context of that configuration.

GardenIndex and GroupIndex values are relative to their local configuration.
The corresponding configuration must be used when resolving these values
to actual Gardens and Groups.

These are the call graphs that have to be adjusted to support graft evaluation.

    eval::value():
        src/eval.rs:
            [ ] environment() ->
            [ ] multi_variable() ->
            [ ] tree_value() ->
            [ ] value()
    eval::environment():
        src/cmd.rs:
            [ ] exec_in_context() ->
            [ ] environment() -> ...
        src/cmds/cmd.rs:
            [ ] cmd() ->
            [ ] environment()
    cmd::exec_in_context():
        src/cmd.rs:
            [ ] exec_in_context() ->
            [ ] environment() -> ...
        src/cmds/exec.rs:
            [ ] exec() ->
            [ ] exec_in_context() ->
            [ ] environment() -> ...
        src/cmds/exec.rs:
            [ ] exec() ->
            [ ] exec_in_context() ->
            [ ] environment() -. ...
        src/cmds/shell.rs:
            [ ] main() ->
            [ ] exec_in_context() ->
            [ ] environment() -. ...
