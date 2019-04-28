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
        app: [libs::lib1, libs::lib2, backend, frontend]


  The trees, gardens, and groups from lib/garden.yaml can be accessed via
  custom::tree custom::group custom::garden.


- config::reader should read Vec<Graft>
