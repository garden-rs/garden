extern crate argparse;
extern crate garden;

use garden::cmd;
use garden::cmds;
use garden::config;
use garden::model;


fn main() {
    std::process::exit(cmd_main());
}


fn cmd_main() -> i32 {
    let mut options = parse_args();

    // The following commands run without a configuration file
    match options.subcommand {
        model::Command::Help => {
            return cmds::help::main(&mut options);
        },
        model::Command::Init => {
            return cmds::init::main(&mut options);
        },
        _ => (),
    }

    let config = config::from_options(&options);
    let mut app = model::ApplicationContext::new(config, options);

    match app.options.subcommand.clone() {
        model::Command::Add => cmds::add::main(&mut app),
        model::Command::Cmd => cmds::cmd::main(&mut app),
        model::Command::Custom(cmd) => cmds::cmd::custom(&mut app, &cmd),
        model::Command::Exec => cmds::exec::main(&mut app),
        model::Command::Eval => cmds::eval::main(&mut app),
        model::Command::Grow => cmds::grow::main(&mut app),
        model::Command::Help => cmd::ExitCode::Success.into(),  // Handled above
        model::Command::Init => cmd::ExitCode::Success.into(),  // Handled above
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

        ap.refer(&mut options.filename_str)
            .add_option(&["-c", "--config"], argparse::Store,
                        "set the config file to use");

        ap.refer(&mut options.chdir)
            .add_option(&["-C", "--chdir"], argparse::Store,
                        "chdir before searching for configuration");

        ap.refer(&mut options.color_mode)
            .add_option(&["--color"], argparse::Store, &color_help);

        ap.refer(&mut options.debug)
            .add_option(&["-d", "--debug"], argparse::Collect,
                        "enable debug categories");

        ap.refer(&mut options.root)
            .add_option(&["-r", "--root"], argparse::Store,
                        "set the garden tree root (${GARDEN_ROOT})");

        ap.refer(&mut options.variables)
            .add_option(&["-s", "--set"], argparse::Collect,
                        "set variables using name=value tokens");

        ap.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"], argparse::StoreTrue,
                        "be verbose");

        ap.refer(&mut options.quiet)
            .add_option(&["-q", "--quiet"], argparse::StoreTrue, "be quiet");

        ap.refer(&mut options.subcommand).required()
            .add_argument("command", argparse::Store,
                          "{add, cmd, eval, exec, ls, shell, <custom>}");

        ap.refer(&mut options.args)
            .add_argument("arguments", argparse::List, "command arguments");

        ap.parse_args_or_exit();
    }
    options.update();

    options
}
