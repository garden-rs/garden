extern crate argparse;
extern crate garden;

use garden::cmds;
use garden::model;


fn main() {
    let mut options = model::CommandOptions::default();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden - git tree organizer");

        ap.refer(&mut options.filename_str)
            .add_option(&["-c", "--config"], argparse::Store,
                        "specify the config file to use");

        ap.refer(&mut options.debug_str)
            .add_option(&["-d", "--debug"], argparse::Store,
                        "debug categories to enable (comma-separated)");

        ap.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"],
                        argparse::StoreTrue, "be verbose");

        ap.refer(&mut options.quiet)
            .add_option(&["-q", "--quiet"],
                        argparse::StoreTrue, "be quiet");

        ap.refer(&mut options.subcommand).required()
            .add_argument("command", argparse::Store,
                "command to run {cmd, exec, help, ls, init, shell, status}");

        ap.refer(&mut options.args)
            .add_argument("arguments", argparse::List,
                "sub-command arguments");

        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    options.update();

    let subcommand = options.subcommand.clone();
    match subcommand {
        model::Command::Add => cmds::help::main(&mut options),
        model::Command::Help => cmds::help::main(&mut options),
        model::Command::Cmd => cmds::cmd::main(&mut options),
        model::Command::Custom(cmd) => cmds::cmd::custom(&mut options, &cmd),
        model::Command::Exec => cmds::exec::main(&mut options),
        model::Command::Eval => cmds::eval::main(&mut options),
        model::Command::Init => cmds::help::main(&mut options),
        model::Command::List => cmds::list::main(&mut options),
        model::Command::Shell => cmds::help::main(&mut options),
    }
}
