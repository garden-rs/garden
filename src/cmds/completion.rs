use anyhow::Result;
use clap::{value_parser, Arg, Command, CommandFactory, Parser};
use clap_complete;
use clap_complete::Shell;
use std::io::Write;

use super::super::cli::MainOptions;
use super::super::config;

/// Generate shell completions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct CompletionOptions {
    /// Include completions for custom commands
    #[arg(long, short)]
    commands: bool,
    /// Shell syntax to emit
    #[arg(default_value_t = Shell::Bash, value_parser = value_parser!(Shell))]
    pub shell: Shell,
}

/// Print shell completions for "garden completion"
pub fn main(options: &MainOptions, completion_options: &CompletionOptions) -> Result<()> {
    let mut cmd = MainOptions::command();

    // Register custom commands with the completion system
    if completion_options.commands {
        let config = config::from_options(options)?;
        for name in config.commands.keys() {
            cmd = cmd.subcommand(
                Command::new(name)
                    .about(format!("Custom {name} command"))
                    .arg(
                        Arg::new("keep_going")
                            .help("Continue to the next tree when errors occur")
                            .short('k')
                            .long("keep-going"),
                    )
                    .arg(
                        Arg::new("no_errexit")
                            .help("Do not pass -e to the shell")
                            .short('n')
                            .long("no-errexit"),
                    )
                    .arg(
                        Arg::new("queries")
                            // NOTE: value_terminator may not be needed in future versions of clap_complete.
                            // https://github.com/clap-rs/clap/pull/4612
                            .value_terminator("--")
                            .help("Tree queries to find trees where commands will be run"),
                    )
                    .arg(
                        Arg::new("arguments")
                            .help("Arguments to forward to custom commands")
                            .last(true),
                    ),
            );
        }
    }

    let mut buf = vec![];
    clap_complete::generate(completion_options.shell, &mut cmd, "garden", &mut buf);
    std::io::stdout().write_all(&buf).unwrap_or(());

    Ok(())
}
