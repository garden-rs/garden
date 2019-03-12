#!/bin/sh
rm -rf examples &&
rm -rf repos &&
mkdir -p repos &&
# Create repos/example.git
git init --bare repos/example.git && (
    # Create an empty commit
    cd ./repos/example.git &&
    tree=$(git write-tree)
    git commit-tree -m 'initial commit' "$tree" >refs/heads/master
)
