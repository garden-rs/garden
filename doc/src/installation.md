# Installation

There are multiple ways to install garden.

These instructions assume that you have `cargo` installed for Rust development.

## Rust and Cargo

If you already have `cargo` installed then you can skip this section.

You may be able to install `cargo` on macOS and Linux using standard package
managers, eg. `brew install rust` or `apt install rust-all`.

Other platforms and older distributions can get a Rust development toolchain
by going to [rustup.rs and following the installation instructions](https://rustup.rs).

## Crates.io

This requires at least Rust 1.45 and Cargo to be installed. Once you have
installed Rust, type the following in the terminal:

```
cargo install garden-tools
```

This will download and compile garden for you. The only thing left to do is
to add the Cargo `$HOME/.cargo/bin` directory to your `$PATH`.

## Latest using Cargo

The version published to crates.io will sometimes be behind the source
code repository. If you want to install the latest pre-release version then you can
build the Git version of Garden yourself using `cargo`.

```
cargo install --git https://github.com/davvid/garden garden-tools
```

## Build and Install from Source for Development

If you would like to develop features and contribute to Garden then you will
have to clone the repository on your local machine.

```
git clone https://github.com/davvid/garden.git
cd garden

# Build ./target/debug/garden
cargo build

# Install $HOME/.cargo/bin/garden
cargo install --path .
```

A `Makefile` is also provided with a classic `make install` installation target.
The `prefix` and `DESTDIR` variables specify the
[installation prefix](https://www.gnu.org/prep/standards/html_node/Directory-Variables.html#Directory-Variables)
and optional
[temporary staging directory](https://www.gnu.org/prep/standards/html_node/DESTDIR.html#DESTDIR),
respectively.

    make install prefix=/usr/local DESTDIR=/tmp/stage

Running `make install` with no arguments installs to `~/.cargo/bin/garden` by default.

`make test` runs the test suite and `make check` runs checks and linters.
