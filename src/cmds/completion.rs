use std::io::Write;

use anyhow::Result;
use clap::{value_parser, CommandFactory, Parser};
use clap_complete;
use clap_complete::Shell;

use super::super::cli::MainOptions;

/// Generate shell completions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct CompletionOptions {
    /// Shell syntax to emit
    #[arg(default_value_t = Shell::Bash, value_parser = value_parser!(Shell))]
    pub shell: Shell,
}

/// Print shell completions for "garden completion"
pub fn main(shell: Shell) -> Result<()> {
    let mut cmd = MainOptions::command();
    let mut buf = vec![];
    clap_complete::generate(shell, &mut cmd, "garden", &mut buf);
    std::io::stdout().write_all(&buf).unwrap_or(());

    Ok(())
}
