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
        for custom_cmd in config.commands {
            cmd = cmd.subcommand(
                Command::new(custom_cmd.get_name())
                    .about(format!("Custom {} command", custom_cmd.get_name()))
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
