use anyhow::Result;
use clap::Parser;

use garden::{cli, cmds, errors, model};

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

    let app = model::ApplicationContext::from_options(&options)?;
    match options.command {
        cli::Command::Cmd(mut cmd) => cmds::cmd::main_cmd(&app, &mut cmd),
        cli::Command::Completion(_) => Ok(()), // Handled above
        cli::Command::Custom(args) => cmds::cmd::main_custom(&app, &args),
        cli::Command::Eval(eval) => cmds::eval::main(&app, &eval),
        cli::Command::Exec(mut exec) => cmds::exec::main(&app, &mut exec),
        cli::Command::Grow(grow) => cmds::grow::main(&app, &grow),
        #[cfg(feature = "gui")]
        cli::Command::Gui(gui) => cmds::gui::main(&app, &gui),
        cli::Command::Init(_) => Ok(()), // Handled above
        cli::Command::List(mut list) => cmds::list::main(&app, &mut list),
        cli::Command::Plant(plant) => cmds::plant::main(&app, &plant),
        cli::Command::Prune(mut prune) => cmds::prune::main(&app, &mut prune),
        cli::Command::Shell(shell) => cmds::shell::main(&app, &shell),
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
