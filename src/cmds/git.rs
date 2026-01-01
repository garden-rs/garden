use anyhow::Result;
use clap::{Parser, ValueHint};

use crate::cmds::exec;
use crate::model;

/// Evaluate garden expressions
#[derive(Parser, Clone, Debug)]
#[command(author, about, long_about)]
pub struct GitOptions {
    /// Filter trees by name post-query using a glob pattern
    #[arg(long, short, default_value = "*")]
    trees: String,
    /// Perform a trial run without executing any commands
    #[arg(long, short = 'N', short_alias = 'n')]
    dry_run: bool,
    /// Run commands in parallel using the specified number of jobs.
    #[arg(long = "jobs", short = 'j', value_name = "JOBS")]
    num_jobs: Option<usize>,
    /// Be quiet
    #[arg(short, long)]
    quiet: bool,
    /// Increase verbosity level (default: 0)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Tree query for the gardens, groups or trees to run the command
    #[arg(default_value = "@*", value_hint=ValueHint::Other)]
    query: String,
    /// Git command to run in the resolved environments
    #[arg(
        default_value = "status",
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    command: Vec<String>,
}

/// Convert GitOptions into ExecOptions
impl From<GitOptions> for exec::ExecOptions {
    fn from(git_options: GitOptions) -> Self {
        exec::ExecOptions {
            dry_run: git_options.dry_run,
            num_jobs: git_options.num_jobs,
            quiet: git_options.quiet,
            verbose: git_options.verbose,
            query: git_options.query,
            command: {
                let mut cmd = vec!["git".to_string()];
                cmd.extend(git_options.command);
                cmd
            },
            trees: git_options.trees,
        }
    }
}

/// Main entry point for the "garden git" command
pub fn main(app_context: &model::ApplicationContext, git_options: &mut GitOptions) -> Result<()> {
    let mut exec_options: exec::ExecOptions = git_options.clone().into();

    exec::main(app_context, &mut exec_options)
}
