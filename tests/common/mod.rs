use garden::cmd;
use garden::config;
use garden::errors;
use garden::model;
use garden::string;

use anyhow::Result;
use assert_cmd::prelude::CommandCargoExt;

use std::process::Command;

fn initialize_environment() {
    // Simplify testing by using a canned environment.
    std::env::set_var("HOME", "/home/test");
    std::env::set_var("PATH", "/usr/bin:/bin");
    std::env::set_var("EMPTY", "");
    std::env::remove_var("PYTHONPATH");
}

pub fn from_string(string: &str) -> model::Configuration {
    initialize_environment();

    let mut config = model::Configuration::new();
    config::parse(string, 0, &mut config).unwrap_or(());

    config
}

pub fn garden_config() -> garden::model::Configuration {
    let string = string!(
        r#"
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
                    - ${prefix}/bin
                    - ${TREE_PATH}/bin
                PYTHONPATH: ${GARDEN_ROOT}/python/send2trash
            commands:
                test:
                    - git status --short
                    - make tox
            remotes:
                davvid: git@github.com:davvid/git-cola.git
        python/qtpy:
            url: https://github.com/spider-ide/qtpy.git
            templates: python
        tmp:
            environment:
                EMPTY: [a, b]
                ${TREE_NAME}_VALUE=: ${TREE_PATH}

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
        oneline: git@example.com:example/oneline.git

    groups:
        cola: [git, cola, python/qtpy]
        test: [a, b, c]
        reverse: [cola, git]
        annex: annex/*
        annex-1: annex/data
        annex-2: annex/local

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
        annex/group:
            groups: annex
        annex/wildcard-groups:
            groups: annex-*
        annex/wildcard-trees:
            trees: annex/*
    "#
    );
    from_string(&string)
}

/// Execute the "garden" command with the specified arguments.
pub fn exec_garden(args: &[&str]) -> Result<()> {
    let mut exec = Command::cargo_bin("garden").expect("garden not found");
    exec.args(args);

    assert!(exec.status().expect("garden returned an error").success());
    Ok(())
}

/// Execute a command and ensure that exit status 0 is returned.
/// Return the captured stdout value as a string.
pub fn garden_capture(args: &[&str]) -> String {
    let mut exec = Command::cargo_bin("garden").expect("garden not found");
    exec.args(args);

    let capture = exec.output();
    assert!(capture.is_ok());

    let utf8_result = String::from_utf8(capture.unwrap().stdout);
    assert!(utf8_result.is_ok());

    utf8_result.unwrap().trim_end().into()
}

/// Execute a command and ensure that the exit status is returned.
pub fn assert_cmd_status(cmd: &[&str], directory: &str, status: i32) {
    let exec = cmd::exec_in_dir(cmd, directory);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());

    match capture.unwrap().exit_status {
        subprocess::ExitStatus::Exited(val) => {
            assert_eq!(val as i32, status);
        }
        subprocess::ExitStatus::Signaled(val) => {
            assert_eq!(val as i32, status);
        }
        subprocess::ExitStatus::Other(val) => {
            assert_eq!(val, status);
        }
        subprocess::ExitStatus::Undetermined => {
            panic!("undetermined exit status");
        }
    }
}

/// Execute a command and ensure that exit status 0 is returned.
pub fn assert_cmd(cmd: &[&str], directory: &str) {
    assert_cmd_status(cmd, directory, errors::EX_OK);
}

/// Execute a command and ensure that exit status 0 is returned. Return the Exec object.
pub fn assert_cmd_capture(cmd: &[&str], directory: &str) -> String {
    let exec = cmd::exec_in_dir(cmd, directory);
    let capture = cmd::capture_stdout(exec);
    assert!(capture.is_ok());

    cmd::trim_stdout(&capture.unwrap())
}

/// Assert that the specified path exists.
pub fn assert_path(path: &str) {
    let pathbuf = std::path::PathBuf::from(path);
    assert!(pathbuf.exists());
}

/// Assert that the specified path is a Git worktree.
pub fn assert_git_worktree(path: &str) {
    assert_path(&format!("{path}/.git"));
}

/// Assert that the Git ref exists in the specified repository.
pub fn assert_ref(repository: &str, refname: &str) {
    let cmd = ["git", "rev-parse", "--quiet", "--verify", refname];
    assert_cmd(&cmd, repository);
}

/// Assert that the Git ref does not exist in the specified repository.
pub fn assert_ref_missing(repository: &str, refname: &str) {
    let cmd = ["git", "rev-parse", "--quiet", "--verify", refname];
    assert_cmd_status(&cmd, repository, 1);
}

/// Cleanup and create a bare repository for cloning
fn setup_tmp_bare_repo(name: &str, path: &str) {
    let cmd = ["../integration/setup.sh", name];
    assert_cmd(&cmd, path);
}

fn teardown_tmp_test_data(path: &str) {
    if let Err(err) = std::fs::remove_dir_all(path) {
        panic!("unable to remove '{path}': {err}");
    }
}

/// Provide a bare repository fixture for the current test.
pub struct BareRepoFixture<'a> {
    name: &'a str,
}

impl<'a> BareRepoFixture<'a> {
    /// Create the test bare repository.
    pub fn new(name: &'a str) -> Self {
        setup_tmp_bare_repo(name, "tests/tmp");

        Self { name }
    }

    /// Return the temporary directory for the current test.
    pub fn root(&self) -> String {
        format!("tests/tmp/{}", self.name)
    }

    /// Return a pathbuf
    pub fn root_pathbuf(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(self.root())
    }

    /// Return a path relative to the temporary directory for the current test.
    pub fn path(&self, path: &str) -> String {
        let fixture_path = format!("{}/{}", self.root(), path);
        assert_path(&fixture_path);

        fixture_path
    }

    /// Return a PathBuf relative to the temporary directory for the current test.
    pub fn pathbuf(&self, path: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(self.path(path))
    }

    /// Asserts that the path is a Git worktree.
    /// Returns the path to the specified worktree.
    pub fn worktree(&self, path: &str) -> String {
        let worktree = self.path(path);
        assert_git_worktree(&worktree);

        worktree
    }

    /// Asserts that the path is a Git worktree.
    /// Returns a pathbuf for the specified worktree.
    pub fn worktree_pathbuf(&self, path: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(self.worktree(path))
    }
}

impl Drop for BareRepoFixture<'_> {
    /// Teardown the test repository.
    fn drop(&mut self) {
        teardown_tmp_test_data(&self.root());
    }
}
