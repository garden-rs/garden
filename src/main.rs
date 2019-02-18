extern crate argparse;
extern crate glob;
extern crate subprocess;

#[macro_use]
extern crate garden;

use garden::model;
use garden::cmds;


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
    let mut config = garden::config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", config);
    }
    if options.is_debug("cmd") {
        debug!("subcommand: cmd");
        debug!("expr: {}", expr);
        debug!("commands: {:?}", commands);
    }

    garden::cmds::cmd::main(
        &mut config, options.quiet, options.verbose,
        options.keep_going, expr, &commands);
}


fn garden_exec(options: &mut model::CommandOptions) {
    options.args.insert(0, "garden exec".to_string());

    let mut expr = String::new();
    let mut command: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden exec - run commands inside gardens");

        ap.refer(&mut expr).required()
            .add_argument("tree-expr", argparse::Store,
                          "gardens/trees to exec (tree expression)");

        ap.refer(&mut command).required()
            .add_argument("command", argparse::List,
                          "command to run over resolved trees");

        ap.stop_on_first_argument(true);
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    let verbose = options.is_debug("config::new");
    let mut config = garden::config::new(&options.filename, verbose);
    if options.is_debug("config") {
        debug!("{}", config);
    }
    if options.is_debug("exec") {
        debug!("subcommand: exec");
        debug!("expr: {}", expr);
        debug!("command: {:?}", command);
    }

    garden::cmds::exec::main(
        &mut config, options.quiet, options.verbose, expr, &command);
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
        model::Command::Exec => garden_exec(&mut options),
        model::Command::Eval => cmds::evaluate::main(&mut options),
        model::Command::Init => cmds::help::main(&mut options),
        model::Command::List => cmds::list::main(&mut options),
        model::Command::Status => cmds::help::main(&mut options),
        model::Command::Shell => cmds::help::main(&mut options),
    }
}
