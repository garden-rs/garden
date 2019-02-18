extern crate argparse;
extern crate glob;
extern crate subprocess;

#[macro_use]
extern crate garden;

use garden::cmds;
use garden::config;
use garden::model;


fn garden_cmd(options: &mut model::CommandOptions) {
    options.args.insert(0, "garden run".to_string());

    let mut expr = String::new();
    let mut commands: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden cmd - run preset commands over gardens");

        ap.refer(&mut options.keep_going)
            .add_option(&["-k", "--keep-going"], argparse::StoreTrue,
                        "continue to the next tree when errors occur");

        ap.refer(&mut expr).required()
            .add_argument("tree-expr", argparse::Store,
                          "gardens/trees to exec (tree expression)");

        ap.refer(&mut commands).required()
            .add_argument("commands", argparse::List,
                          "commands to run over resolved trees");

        ap.stop_on_first_argument(true);
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let verbose = options.is_debug("config::new");
    let mut cfg = config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", cfg);
    }
    if options.is_debug("cmd") {
        debug!("subcommand: cmd");
        debug!("expr: {}", expr);
        debug!("commands: {:?}", commands);
    }

    let quiet = options.quiet;
    let verbose = options.verbose;
    let keep_going = options.keep_going;
    cmds::cmd::main(&mut cfg, quiet, verbose, keep_going, expr, &commands);
}


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

    match options.subcommand {
        model::Command::Add => cmds::help::main(&mut options),
        model::Command::Help => cmds::help::main(&mut options),
        model::Command::Cmd => garden_cmd(&mut options),
        model::Command::Exec => cmds::exec::main(&mut options),
        model::Command::Eval => cmds::evaluate::main(&mut options),
        model::Command::Init => cmds::help::main(&mut options),
        model::Command::List => cmds::list::main(&mut options),
        model::Command::Status => cmds::help::main(&mut options),
        model::Command::Shell => cmds::help::main(&mut options),
    }
}
