use ::cmd;
use ::model;


/// Entry point for `garden help`
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(options: &mut model::CommandOptions) {
    let cmd_path = match std::env::current_exe() {
        Err(_) => std::path::PathBuf::from("garden"),
        Ok(path) => path,
    };
    let mut help_cmd = vec!(cmd_path);

    let mut cmd_name = String::new();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden help - command documentation");

        ap.refer(&mut cmd_name)
            .add_argument("command", argparse::Store,
                          "Command help to display");

        options.args.insert(0, "garden help".to_string());
        ap.parse(options.args.to_vec(),
                 &mut std::io::stdout(), &mut std::io::stderr())
            .map_err(|c| std::process::exit(c)).ok();
    }

    // garden help foo -> garden foo --help
    if !cmd_name.is_empty() {
        help_cmd.push(std::path::PathBuf::from(&cmd_name));
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
