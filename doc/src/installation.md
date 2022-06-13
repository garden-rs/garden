# Installation

There are multiple ways to install garden.

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
  git clone https://github.com/davvid/garden
  cd garden

  # Build ./target/debug/garden
  cargo build

  # Install $HOME/.cargo/bin/garden
  cargo install
  ```
