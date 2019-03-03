extern crate garden;


pub fn from_string(string: &str) -> garden::model::Configuration {
    // Simplify testing by using a canned environment.
    std::env::set_var("HOME", "/home/test");
    std::env::set_var("PATH", "/usr/bin:/bin");
    std::env::remove_var("PYTHONPATH");

    let mut config = garden::model::Configuration::new();
    garden::config::parse(string, false, &mut config);

    return config;
}


pub fn garden_config() -> garden::model::Configuration {
    let string = r#"
    garden:
        root: ${root}

    variables:
        echo_cmd: echo cmd
        echo_cmd_exec: $ ${echo_cmd}
        test: TEST
        local: ${test}/local
        src: src
        root: ~/${src}

    templates:
        makefile:
            variables:
                prefix: ${TREE_PATH}/local
            commands:
                build: make -j prefix=${prefix} all
                install: make -j prefix=${prefix} install
                test: make test
        python:
            environment:
                PYTHONPATH: ${TREE_PATH}
        local:
            url: ${local}/${TREE_NAME}
    trees:
        git:
            url: https://github.com/git/git
            templates: makefile
            variables:
                prefix: ~/.local
            gitconfig:
                user.name: A U Thor
                user.email: author@example.com
        cola:
            url: https://github.com/git-cola/git-cola
            path: git-cola
            templates: [makefile, python]
            variables:
                prefix: ${TREE_PATH}/local
            environment:
                PATH:
                    - ${TREE_PATH}/bin
                    - ${prefix}/bin
                PYTHONPATH: ${GARDEN_ROOT}/python/qtpy
            commands:
                test:
                    - git status --short
                    - make tox
            remotes:
                davvid: git@github.com:davvid/git-cola.git
        tmp:
            path: /tmp
            templates: local

        annex/data:
            url: git@example.com:git-annex/data.git
            gitconfig:
                remote.origin.annex-ignore: true
            remotes:
                local: ${GARDEN_ROOT}/annex/local

        annex/local:
            extend: annex/data

    groups:
        cola: [git, qtpy, cola]
        test: [a, b, c]
        reverse: [cola, git]

    gardens:
        cola:
            groups: cola
            variables:
                prefix: ~/apps/git-cola/current
            environment:
                GIT_COLA_TRACE=: full
                PATH+: ${prefix}/bin
            commands:
                summary:
                    - git branch
                    - git status --short
        git:
            groups: cola
            trees: gitk
            gitconfig:
                user.name: A U Thor
                user.email: author@example.com
    "#.to_string();

    return from_string(&string);
}
