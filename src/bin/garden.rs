use anyhow::Result;
use clap::Parser;

use garden::build;
use garden::cli;
use garden::cmds;
use garden::config;
use garden::errors;

/// Main entry point for the "garden" command.
fn main() -> Result<()> {
    // Return the appropriate exit code when a GardenError is encountered.
    if let Err(err) = cmd_main() {
        let exit_status = exit_status_from_error(err);
        std::process::exit(exit_status);
    }

    Ok(())
}

/// Parse command-line options and delegate to the command implementation
fn cmd_main() -> Result<()> {
    let mut options = cli::MainOptions::parse();
    options.update();

    // Handle the "completion" and "init" commands before building the context.
    match options.command.clone() {
        cli::Command::Completion(completion) => {
            return cmds::completion::main(&options, &completion);
        }
        cli::Command::Init(mut init_options) => {
            return cmds::init::main(&options, &mut init_options);
        }
        _ => (), // Handled below
    }

    let config = config::from_options(&options)?;
    let mut app = build::context_from_config(config, &options)?;

    match options.command.clone() {
        cli::Command::Cmd(cmd) => cmds::cmd::main_cmd(&mut app, &cmd),
        cli::Command::Completion(_) => Ok(()), // Handled above
        cli::Command::Custom(args) => cmds::cmd::main_custom(&mut app, &args),
        cli::Command::Eval(eval) => cmds::eval::main(&mut app, &eval),
        cli::Command::Exec(exec) => cmds::exec::main(&mut app, &exec),
        cli::Command::Grow(grow) => cmds::grow::main(&mut app, &grow),
        cli::Command::Init(_) => Ok(()), // Handled above
        cli::Command::Inspect(mut inspect) => cmds::inspect::main(&mut app, &mut inspect),
        cli::Command::List(list) => cmds::list::main(&mut app, &list),
        cli::Command::Plant(plant) => cmds::plant::main(&mut app, &plant),
        cli::Command::Prune(mut prune) => cmds::prune::main(&mut app, &mut prune),
        cli::Command::Shell(shell) => cmds::shell::main(&mut app, &shell),
    }
}

/// Transform an anyhow::Error into an exit code when an error occurs.
fn exit_status_from_error(err: anyhow::Error) -> i32 {
    match err.downcast::<errors::GardenError>() {
        Ok(garden_err) => {
            match garden_err {
                // ExitStatus exits without printing a message.
                errors::GardenError::ExitStatus(status) => status,
                // Other GardenError variants print a message before exiting.
                _ => {
                    eprintln!("error: {garden_err:#}");
                    garden_err.into()
                }
            }
        }
        Err(other_err) => {
            eprintln!("error: {other_err:#}");
            1
        }
    }
}
