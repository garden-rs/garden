# Git Cola Development

This garden file sets up a development garden with the latest version of Git, Git Cola,
qtpy and vx.

Run the following commands to see this in action.

```bash
# Create a directory we'll clone and build a few repositories.
mkdir -p cola && cd cola

# Download and audit the garden file we're going to run.
wget https://gitlab.com/garden-rs/garden/-/raw/main/doc/src/examples/git-cola/garden.yaml
cat garden.yaml

# One-time setup: Clone all of the repos in the "cola" garden and use a custom
# "garden grow" is a garden built-in command that clones.
garden grow cola

# "garden setup" command defined in "garden.yaml" to initializes the environment.
garden setup cola

# All set! Now we can run Git Cola from the development environment.
garden run

# Daily development workflow: run tests in each repository in-place.
garden test cola

# Commands can be passed to the underlying "run" command to run Git Cola
# against any Git repository.
garden run -- --repo path/to/any/git/tree

# These projects don't need to be "built", so this is technically a no-op.
# A Rust or C++ Rust project could use something like this to run "make"
# in each repository.
garden build cola
```

The development repositories are now in your current directory and a
development virtualenv is present in the `./dist` directory.

## garden.yaml

The following is the contents of the `garden.yaml` file used in this example.

```yaml
{{#include git-cola/garden.yaml}}
```

## Pre-defined Custom Commands and Ad-Hoc Commands

Included in `garden.yaml` are a few few helpful commands that give us a quick
view of what's going on in each tree:

```bash
garden diff cola
garden status cola
garden lol cola
```

If we want to perform git stuff (like fetch the latest changes), we can
always use `garden exec` to run arbitrary commands:

```bash
garden exec cola git fetch --verbose

# When needed, we can hop into a shell with all of the environment variables set
garden shell cola
```

## Self-contained installation demo

The `garden run` example runs `git` and `git cola` in-place in their
respective trees. The `git-cola` project is not installed into the `./dist` directory.
It contains just the virtualenv created needed to run it.

In order to create a self-contained installation to run the tools
independently of their source repositories we have to install them into the
`./dist` directory.

The following example installs Git and Git Cola into the `./dist` directory
by running the "make install" targets in each repo:

```bash
garden install cola
```

Now we can test the installed tools directly by adding `./dist/bin` to our
`$PATH`, or just invoke the script directly:

```bash
./dist/bin/git-cola
```

Voila, we now have a fully functional development environment with PyQt5, qtpy
and Git Cola ready to go for development.
