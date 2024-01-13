#!/bin/sh
# export GARDEN_TEST_GIT_VERBOSE=1 to increase verbosity.

main () {
    set -e
    if test -n "${GARDEN_TEST_GIT_VERBOSE}"
    then
        set -x
    fi

    if test -z "$1"
    then
        echo 1>&2 error: subdirectory name is required
        exit 1
    fi

    quiet=${GARDEN_TEST_GIT_VERBOSE:+"--quiet"}

    # Create the test subdirectory
    mkdir -p "$1"
    (
        cd "./$1"
        git init --quiet
        git config user.name Garden
        git config user.email garden-tools@crates.io
        git config init.defaultBranch garden
        git commit --quiet --allow-empty -m'Root directory'
        mkdir -p repos
        # Create repos/example.git
        git init ${quiet} --bare repos/example.git
        (
            # Create an empty commit
            cd ./repos/example.git

            tree=$(git write-tree)
            git commit-tree -m "$1" "$tree" >refs/heads/default
            git symbolic-ref HEAD refs/heads/default
            git commit-tree -m "$1 commit 2" -p "$(git rev-parse HEAD)" "$tree" >refs/heads/default
            git rev-parse HEAD >refs/heads/dev
        )
    )
}

main "$@"
