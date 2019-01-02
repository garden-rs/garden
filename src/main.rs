extern crate argparse;
extern crate subprocess;

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
    type Err = String;
    fn from_str(src: &str) -> Result<Command, String> {
        return match src {
            "add" => Ok(Command::add),
            "ex" => Ok(Command::exec),
            "exec" => Ok(Command::exec),
            "help" => Ok(Command::help),
            "init" => Ok(Command::init),
            "sh" => Ok(Command::shell),
            "shell" => Ok(Command::shell),
            "st" => Ok(Command::status),
            "status" => Ok(Command::status),
            _ => Err(format!("invalid command: {}", src)),
        }
    }
}


fn error(args: std::fmt::Arguments) {
    eprintln!("error: {}", args);
    std::process::exit(1);
}

fn debug(args: std::fmt::Arguments) {
    eprintln!("debug: {}", args);
}


fn garden_help(verbose: bool, args: Vec<String>) {

    let mut cmd_name = String::from("garden");

    match std::env::current_exe() {
        Ok(exe) => {
            cmd_name = exe.to_string_lossy().to_string();
        }
        Err(err) => {
            error(format_args!("failed to get current exe: {}", err));
        }
    }

    let mut help_cmd = Vec::new();
    help_cmd.push(cmd_name.to_string());

    // garden help foo -> garden foo --help
    if args.len() > 0 {
        help_cmd.push(args[0].to_string());
    }

    help_cmd.push("--help".to_string());

    if verbose {
        debug(format_args!("help command"));
        let mut i: i32 = 0;
        for arg in &help_cmd {
            debug(format_args!("help_cmd[{:02}] = {}", i, arg));
            i += 1;
        }
    }

    let mut p = subprocess::Popen::create(
        &help_cmd, subprocess::PopenConfig::default()).unwrap();

    match p.wait() {
        Ok(_) => { }
        Err(_) => { std::process::exit(1); }
    }
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
    let mut command: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden exec - execute commands inside gardens");

        ap.refer(&mut name).required()
            .add_argument("name", argparse::Store, r#"Garden to enter"#);

        ap.refer(&mut command).required()
            .add_argument("command", argparse::List, r#"Command to run"#);

        ap.stop_on_first_argument(true);
        match ap.parse(args, &mut std::io::stdout(), &mut std::io::stderr()) {
            Ok(()) => {}
            Err(err) => {
                std::process::exit(err);
            }
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
