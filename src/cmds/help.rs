use super::super::model;
use super::super::cmd;


/// Entry point for `garden help`
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(options: &mut model::CommandOptions) {
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

    std::process::exit(cmd::run(&help_cmd));
}
