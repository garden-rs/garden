# Installation

There are multiple ways to install `garden`.

These instructions assume that you have `cargo` installed for Rust development.

[Skip ahead to the Homebrew section](#install-using-homebrew) if you're on macOS
and prefer to install `garden` using Homebrew.

[Skip ahead to the NetBSD section](#install-on-netbsd) if you're on NetBSD
and prefer to install `garden` using `pkgin` or the pkgsrc/NetBSD sources.


## Rust and Cargo

If you already have `cargo` installed then you can skip this section.

You may be able to install `cargo` on macOS and Linux using standard package
managers, e.g. `brew install rust` or `apt install rust-all`.

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
cargo install --git https://gitlab.com/garden-rs/garden.git garden-tools
```


## Install using Homebrew

You can install `garden` on macOS using [Homebrew](https://brew.sh/).

### Add the homebrew-garden tap

*NOTE*: The custom `brew tap` will not be needed in the future once the
[Garden repository gets enough stars, forks or watchers](https://github.com/Homebrew/homebrew-core/pull/119195)
to allow `garden` to be added to the main homebrew-core repository.

You will need to enable the `homebrew-garden` tap until then.

```bash
brew tap garden-rs/garden https://gitlab.com/garden-rs/homebrew-garden
```

### Stable Release

To install the latest stable release from the `homebrew-garden` tap:
```bash
brew install garden
```
Upgrade `garden` in the future by using `brew upgrade garden`.

### Development Version

To install the latest development version from Git:
```bash
brew install --head garden
```

*NOTE*: If you install the latest development version with `--head` then you will need to use
`brew upgrade --fetch-HEAD garden`  to upgrade it.

If you don't specify `--fetch-HEAD` when upgrading then `brew upgrade garden` will
actually downgrade `garden` to the latest stable release.

### Cleanup

Installing `garden` with Homebrew may leave behind the Rust development tools.

Use `brew remove rust` after `garden` is installed to save on disk space.

Read on for how to build garden from source.


## Install on NetBSD

Garden has been packaged for
[pkgsrc/NetBSD](http://mail-index.netbsd.org/pkgsrc-changes/2023/01/22/msg267560.html)
as described in [#13](https://github.com/davvid/garden/issues/13).

To install from the official repository, run:

```bash
pkgin install garden
```

If you prefer to build from the pkgsrc sources, run:

```bash
cd /usr/pkgsrc/devel/garden
make install
```


## Build and Install from Source for Development

If you would like to develop features and contribute to Garden then you will
have to clone the repository on your local machine.

```bash
git clone https://gitlab.com/garden-rs/garden.git
cd garden

# Build ./target/debug/garden
cargo build

# Install $HOME/.cargo/bin/garden
cargo install --path .
```

Running `cargo install` with no arguments installs to `~/.cargo/bin/garden` by default.

Once you have `garden` installed then you can use Garden's `garden.yaml` to run
Garden's custom workflow commands.

* `garden test` runs the test suite using `cargo test`.
* `garden check` runs checks and lints.
* `garden doc` builds the documentation.
* `garden fmt` formats the source code.
* `garden install-doc` installs the documentation.

See Garden's `garden.yaml` for more details.


## Windows

Garden is developed on Linux and supported on macOS and BSDs where Rust is available.

Garden is not supported on Windows.

Garden "should" work fine on Windows if you install a shell (e.g. `bash.exe` or
`zsh.exe`) in `$PATH` and patch a few details to deal with Windows-isms, but Garden is
untested and not supported by the core team on Windows or WSL.

Issues related to Windows will be closed. Pull requests related to these systems are
welcome as long as they do not clutter the core or test suite with Windows-isms.
