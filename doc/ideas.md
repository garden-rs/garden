# Ideas, New Features, Usability Enhancements

## Grafts (WIP)

Make it possible to graft configuration from one garden file into another.
Create "grafted" gardens by including an external garden.yaml and making its
trees, variables and groups available under a `<graft>::` namespace that can
be evaluated from the scope of an outer garden file.

Grafts are arbitrarily nested. A config inside a namespace can itself have grafts.
Nested grafts are addressed by using `nested::graft::` namespace prefixes.

    ### top-level garden.yaml
    grafts:
      libs: libs/garden.yaml
      private:
        path: libs/private.yaml
        root: libs/private

    trees:
      server: ${vcs}/server
      client: ${vcs}/client

    groups:
      app: [libs::http, libs::rest, libs::deps::utils, private::auth, server, client]

    # Provide the "vcs" variable used by the grafted "libs" and "private" namespaces.
    variables:
      vcs: ssh://git@private.example.com


    ### libs/garden.yaml
    grafts:
      deps: deps/garden.yaml

    trees:
      http: ${vcs}/http.git
      rest: ${vcs}/rest.git

    groups:
      libs: [deps::utils, http, rest]


    ### libs/deps/garden.yaml
    trees:
      utils: ${vcs}/utils.git


    ### libs/private.yaml
    trees:
      auth: ${vcs}/auth.git


The trees, gardens, and groups from `libs/garden.yaml` can be accessed via
`libs::*`. Its root directory is `libs/` by default unless specified using
the `root` field, as demonstrated by the `private` graft.

When a variable reference contains `ns::nested::` namespace/graft prefixes
then we need to walk down the hierarchy for each level of `::` hierarchy.
We start from the current Configuration and traverse down one level to the immediate
child graft and only consider variables within the final graft's namespace.

When a variable has no namespace specifiers, like the `vcs` varible specified
above, then we start from the Configuration holding the reference and walk up the
parent hierarchy. The first enclosing scope containing the variable is used.

The configuration reader strategy is to first read the top-level configuration.
When a configuration is read, stub Graft entries are recorded in the Configuration's
`grafts` attribute so that they can be populated later.

Once the Configuration has been read, child Configuration grafts are read and
stitched into the parent graft.  At this point the parent's NodeId is recorded
into the child Configuration so that traversal can use this information to
find the parent Configuration when resolving variables.

To support top-down traversal, the child graft Configuration ConfigId is
recorded into the parent Configuration's graft entry.


- Stategy for Tree contexts

When an `Option<ConfigId>` is present in the tree context then the value must
be evaluated using the Configuration corresponding to the ConfigId.
The ConfigId looks up the Configuration for that context.


- Strategy for evaluating values

`query::tree_context(query: String)` resolves a string to a TreeContext that
represents a Tree in a particular garden or grafted configuration.

When the query contains "the-graft::tree" identifiers then we first attempt to
find a graft that matches the name. If a graft named "the-graft" exists then
the child configuration is looked up for "the-graft" and the query is resolved
in the context of that configuration.

GardenName and GroupName values are relative to their local configuration.
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
