# Ideas, New Features, Usability Enhancements

- config::reader should stop creating `${TREE_NAME}`, `${GARDEN_ROOT}`,
  and `${GARDEN_CONFIG_DIR}`.  config.rs should create the NamedVariables.

- Modular/reusable garden.yaml configuration.

Allow "graft" gardens by including the garden.yaml from an external garden.yaml
and placings its tree, groups, and gardens in a "custom::" graft namespace.

    grafts:
        libs: libs/garden.yaml
        deps:
            config: deps/deps.yaml
            root: deps

    trees:
        server:
            url: ${vcs}/server
        client:
            url: ${vcs}/client

    groups:
        app: [libs::lib1, libs::lib2, server, client]


The trees, gardens, and groups from lib/garden.yaml can be accessed via
custom::tree custom::group custom::garden.

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
