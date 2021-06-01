use anyhow::Result;

use garden::build;
use garden::cmds;
use garden::config;
use garden::errors;
use garden::model;

fn main() -> Result<()> {
    // Return the appropriate exit code when a GardenError is encountered.
    if let Err(err) = cmd_main() {
        let exit_status: i32 = match err.downcast::<errors::GardenError>() {
            Ok(garden_err) => {
                match garden_err {
                    // ExitStatus exits without printing a message.
                    errors::GardenError::ExitStatus(status) => status,
                    // Other GardenError variants print a message before exiting.
                    _ => {
                        eprintln!("error: {:#}", garden_err);
                        garden_err.into()
                    }
                }
            }
            Err(other_err) => {
                eprintln!("error: {:#}", other_err);
                1
            }
        };
        std::process::exit(exit_status);
    }

    Ok(())
}

fn cmd_main() -> Result<()> {
    let mut options = parse_args();

    // The following commands run without a configuration file
    match options.subcommand {
        model::Command::Help => {
            return cmds::help::main(&mut options);
        }
        model::Command::Init => {
            return cmds::init::main(&mut options);
        }
        _ => (),
    }

    let config = config::from_options(&options)?;
    let mut app = build::context_from_config(config, options)?;

    match app.options.subcommand.clone() {
        model::Command::Add => cmds::add::main(&mut app),
        model::Command::Cmd => cmds::cmd::main(&mut app),
        model::Command::Custom(cmd) => cmds::cmd::custom(&mut app, &cmd),
        model::Command::Exec => cmds::exec::main(&mut app),
        model::Command::Eval => cmds::eval::main(&mut app),
        model::Command::Grow => cmds::grow::main(&mut app),
        model::Command::Help => Ok(()), // Handled above
        model::Command::Init => Ok(()), // Handled above
        model::Command::Inspect => cmds::inspect::main(&mut app),
        model::Command::List => cmds::list::main(&mut app),
        model::Command::Shell => cmds::shell::main(&mut app),
    }
}

fn parse_args() -> model::CommandOptions {
    let color_names = model::ColorMode::names();
    let color_help = format!("set color mode {{{}}}", color_names);

    let mut options = model::CommandOptions::default();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden - cultivate git trees");
        ap.stop_on_first_argument(true);

        ap.refer(&mut options.filename_str).add_option(
            &["-c", "--config"],
            argparse::Store,
            "set the config file to use",
        );

        ap.refer(&mut options.chdir).add_option(
            &["-C", "--chdir"],
            argparse::Store,
            "chdir before searching for configuration",
        );

        ap.refer(&mut options.color_mode)
            .add_option(&["--color"], argparse::Store, &color_help);

        ap.refer(&mut options.debug).add_option(
            &["-d", "--debug"],
            argparse::Collect,
            "enable debug categories",
        );

        ap.refer(&mut options.root).add_option(
            &["-r", "--root"],
            argparse::Store,
            "set the garden tree root (${GARDEN_ROOT})",
        );

        ap.refer(&mut options.variables).add_option(
            &["-s", "--set"],
            argparse::Collect,
            "set variables using name=value tokens",
        );

        ap.refer(&mut options.verbose).add_option(
            &["-v", "--verbose"],
            argparse::StoreTrue,
            "be verbose",
        );

        ap.refer(&mut options.quiet).add_option(
            &["-q", "--quiet"],
            argparse::StoreTrue,
            "be quiet",
        );

        ap.refer(&mut options.subcommand).required().add_argument(
            "command",
            argparse::Store,
            "{add, cmd, eval, exec, grow, help, init, inspect, ls, shell, <custom>}",
        );

        ap.refer(&mut options.args)
            .add_argument("arguments", argparse::List, "command arguments");

        ap.parse_args_or_exit();
    }
    options.update();

    options
}
