extern crate argparse;
extern crate garden;

use garden::cmds;
use garden::config;
use garden::model;


fn main() {
    let mut options = model::CommandOptions::default();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden - git tree organizer");

        ap.refer(&mut options.filename_str)
            .add_option(&["-c", "--config"], argparse::Store,
                        "specify the config file to use");

        ap.refer(&mut options.debug)
            .add_option(&["-d", "--debug"], argparse::Collect,
                        "enable debug categories");

        ap.refer(&mut options.variables)
            .add_option(&["-s", "--set"], argparse::Collect,
                        "override variables using name=value tokens");

        ap.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"],
                        argparse::StoreTrue, "be verbose");

        ap.refer(&mut options.quiet)
            .add_option(&["-q", "--quiet"], argparse::StoreTrue, "be quiet");

        ap.refer(&mut options.subcommand).required()
            .add_argument("command", argparse::Store, "command to run");

        ap.refer(&mut options.args)
            .add_argument("arguments", argparse::List, "command arguments");

        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }
    options.update();

    let config = config::from_options(&options);
    let mut app = model::ApplicationContext::new(config, options.clone());

    match app.options.subcommand.clone() {
        model::Command::Add => cmds::add::main(&mut options),
        model::Command::Help => cmds::help::main(&mut options),
        model::Command::Cmd => cmds::cmd::main(&mut options),
        model::Command::Custom(cmd) => cmds::cmd::custom(&mut options, &cmd),
        model::Command::Exec => cmds::exec::main(&mut options),
        model::Command::Eval => cmds::eval::main(&mut app),
        model::Command::Init => cmds::help::main(&mut options),
        model::Command::List => cmds::list::main(&mut options),
        model::Command::Shell => cmds::shell::main(&mut options),
    }
}
