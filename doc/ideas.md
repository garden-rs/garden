# Ideas, New Features, Usability Enhancements

- std::io::Result<(), i32> for main()

- Modular/reusable garden.yaml configuration.

Allow "graft" gardens by including the garden.yaml from an external garden.yaml
and placings its tree, groups, and gardens in a "custom::" namespace.

    grafts:
        libs: libs/garden.yaml

    trees:
        server:
            url: ${vcs}/server
        client:
            url: ${vcs}/client

    groups:
        app: [libs::lib1, libs::lib2, backend, frontend]


  The trees, gardens, and groups from lib/garden.yaml can be accessed via
  custom::tree custom::group custom::garden.
