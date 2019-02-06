extern crate argparse;
extern crate subprocess;

#[macro_use]
extern crate garden;


#[derive(Debug)]
enum Command {
    Add,
    Exec,
    Help,
    Init,
    Shell,
    Status,
}

impl std::default::Default for Command {
    fn default() -> Self {
        return Command::Help;
    }
}

impl_display_brief!(Command);


#[derive(Default)]
struct CommandOptions {
    args: Vec<String>,
    debug: std::collections::HashSet<String>,
    debug_str: String,
    filename: Option<std::path::PathBuf>,
    filename_str: String,
    subcommand: Command,
    verbose: bool,
}

impl CommandOptions {
    fn update(&mut self) {
        if self.filename_str.len() > 0 {
            self.filename = Some(std::path::PathBuf::from(&self.filename_str));
        }

        for debug_name in self.debug_str.split(",") {
            self.debug.insert(debug_name.to_string());
        }
    }

    fn is_debug(&self, name: &str) -> bool {
        return self.debug.contains(name);
    }
}


impl std::str::FromStr for Command {
    type Err = ();  // For the FromStr trait

    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "add" => Ok(Command::Add),
            "ex" => Ok(Command::Exec),
            "exec" => Ok(Command::Exec),
            "help" => Ok(Command::Help),
            "init" => Ok(Command::Init),
            "sh" => Ok(Command::Shell),
            "shell" => Ok(Command::Shell),
            "st" => Ok(Command::Status),
            "stat" => Ok(Command::Status),
            "status" => Ok(Command::Status),
            _ => Err(()),
        }
    }
}

fn garden_help(options: &mut CommandOptions) {
    let cmd_path = match std::env::current_exe() {
        Err(_) => std::path::PathBuf::from("garden"),
        Ok(path) => path,
    };
    let mut help_cmd = vec!(cmd_path);

    let mut command = String::new();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden help - command documentation");
        ap.stop_on_first_argument(true);

        ap.refer(&mut command)
            .add_argument("command", argparse::Store,
                          "Command help to display");

        options.args.insert(0, "garden help".to_string());
        ap.parse(options.args.to_vec(),
                 &mut std::io::stdout(), &mut std::io::stderr())
            .map_err(|c| std::process::exit(c)).ok();
    }

    // garden help foo -> garden foo --help
    if !command.is_empty() {
        help_cmd.push(std::path::PathBuf::from(command));
    }

    help_cmd.push(std::path::PathBuf::from("--help"));

    if options.verbose {
        debug!("help command");
        let mut i: i32 = 0;
        for arg in &help_cmd {
            debug!("help_cmd[{:02}] = {:?}", i, arg);
            i += 1;
        }
    }

    std::process::exit(garden::cmd::get_status(&help_cmd));
}

fn garden_exec(options: &mut CommandOptions) {
    options.args.insert(0, "garden exec".to_string());

    let mut trees = String::new();
    let mut command: Vec<String> = Vec::new();

    // Parse arguments
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden exec - run commands inside gardens");

        ap.refer(&mut trees).required()
            .add_argument("trees", argparse::Store,
                          "Gardens/trees to run inside (tree expression)");

        ap.refer(&mut command).required()
            .add_argument("command", argparse::List,
                          "Command to over resolved trees");

        ap.stop_on_first_argument(true);
        if let Err(err) = ap.parse(options.args.to_vec(),
                                   &mut std::io::stdout(),
                                   &mut std::io::stderr()) {
            std::process::exit(err);
        }
    }

    // Resolve garden and tree names into a set of trees
    let verbose = options.is_debug("config::new");
    let config = garden::config::new(&options.filename, verbose);

    if options.is_debug("config") {
        debug!("{}", config);
    }

    // Execute commands for each tree
    if options.is_debug("exec") {
        debug!("subcommand: exec");
        debug!("trees: {}", trees);
        debug!("exec arguments:");
        let mut i: i32 = 0;
        for arg in &command {
            debug!("\targs[{:02}] = {}", i, arg);
            i += 1;
        }
    }
}

fn main() {
    let mut options = CommandOptions::default();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden - git tree organizer");

        ap.refer(&mut options.filename_str)
            .add_option(&["-c", "--config"], argparse::Store,
                        "Specify the garden configuration filename");

        ap.refer(&mut options.debug_str)
            .add_option(&["-d", "--debug"], argparse::Store,
                        "Debug categories to enable (comma-separated)");

        ap.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"],
                        argparse::StoreTrue, "Be verbose");


        ap.refer(&mut options.subcommand).required()
            .add_argument("command", argparse::Store,
                "Command to run {add, exec, help, init, shell, status}");

        ap.refer(&mut options.args)
            .add_argument("arguments", argparse::List,
                "Arguments for sub-command");

        ap.stop_on_first_argument(true);
        ap.parse_args_or_exit();
    }

    options.update();

    match options.subcommand {
        Command::Add => garden_help(&mut options),
        Command::Help => garden_help(&mut options),
        Command::Exec => garden_exec(&mut options),
        Command::Init => garden_help(&mut options),
        Command::Status => garden_help(&mut options),
        Command::Shell => garden_help(&mut options),
    }
}
