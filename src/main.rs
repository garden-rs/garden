extern crate argparse;
extern crate garden;
extern crate subprocess;

use garden::cmd::debug;

#[allow(non_camel_case_types)]
#[derive(Debug)]
enum Command {
    add,
    exec,
    help,
    init,
    shell,
    status,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "add" => Ok(Command::add),
            "ex" => Ok(Command::exec),
            "exec" => Ok(Command::exec),
            "help" => Ok(Command::help),
            "init" => Ok(Command::init),
            "sh" => Ok(Command::shell),
            "shell" => Ok(Command::shell),
            "st" => Ok(Command::status),
            "stat" => Ok(Command::status),
            "status" => Ok(Command::status),
            _ => Err(()),
        }
    }
}

fn garden_help(verbose: bool, args: Vec<String>) {

    let cmd_path = match std::env::current_exe() {
        Err(_) => std::path::PathBuf::from("garden"),
        Ok(path) => path,
    };
    let mut help_cmd = vec!(cmd_path);

    // garden help foo -> garden foo --help
    if args.len() > 0 {
        help_cmd.push(std::path::PathBuf::from(args[0].to_string()));
    }

    help_cmd.push(std::path::PathBuf::from("--help"));

    if verbose {
        debug(format_args!("help command"));
        let mut i: i32 = 0;
        for arg in &help_cmd {
            debug(format_args!("help_cmd[{:02}] = {}",
                               i, arg.to_string_lossy()));
            i += 1;
        }
    }

    std::process::exit(garden::cmd::get_status(&help_cmd));
}

fn garden_exec(verbose: bool, mut args: Vec<String>) {
    args.insert(0, "garden exec".to_string());

    if verbose {
        debug(format_args!("exec arguments"));
        let mut i: i32 = 0;
        for arg in &args {
            debug(format_args!("args[{:02}] = {}", i, arg));
            i += 1;
        }
    }

    let mut name = String::new();
    let mut command: Vec<String> = vec!();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden exec - execute commands inside gardens");

        ap.refer(&mut name).required()
            .add_argument("name", argparse::Store, r#"Garden to enter"#);

        ap.refer(&mut command).required()
            .add_argument("command", argparse::List, r#"Command to run"#);

        ap.stop_on_first_argument(true);
        if let Err(err) = ap.parse(args,
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    // Resolve garden and tree names into a set of trees

    // Execute commands for each tree
}

fn main() {
    let mut verbose = false;
    let mut subcommand = Command::help;
    let mut args = vec!();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden - git tree organizer");
        ap.refer(&mut verbose)
            .add_option(&["-v", "--verbose"],
                        argparse::StoreTrue, "Be verbose");
        ap.refer(&mut subcommand).required()
            .add_argument("command", argparse::Store,
                r#"Command to run {add, exec, help, init, shell, status}"#);
        ap.refer(&mut args)
            .add_argument("arguments", argparse::List,
                r#"Arguments for sub-command"#);
        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    match subcommand {
        Command::add => garden_help(verbose, args),
        Command::help => garden_help(verbose, args),
        Command::exec => garden_exec(verbose, args),
        Command::init => garden_help(verbose, args),
        Command::status => garden_help(verbose, args),
        Command::shell => garden_help(verbose, args),
    }
}
