use anyhow::Result;

use super::super::cmd;
use super::super::model;

/// Entry point for `garden help`
/// Parameters:
/// - options: `garden::model::CommandOptions`

pub fn main(options: &mut model::CommandOptions) -> Result<()> {
    let cmd_path = cmd::current_exe();
    let mut help_cmd = vec![cmd_path];

    let mut cmd_name = String::new();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden help - Display command documentation");

        ap.refer(&mut cmd_name).add_argument(
            "command",
            argparse::Store,
            "{add, cmd, eval, exec, ls, shell}",
        );

        options.args.insert(0, "garden help".into());
        cmd::parse_args(ap, options.args.to_vec());
    }

    // garden help foo -> garden foo --help
    if !cmd_name.is_empty() {
        help_cmd.push(cmd_name);
    }

    help_cmd.push("--help".into());

    if options.verbose > 0 {
        debug!("help command");
        for (i, arg) in help_cmd.iter().enumerate() {
            debug!("help_cmd[{:02}] = {:?}", i, arg);
        }
    }

    cmd::run(&help_cmd).map_err(|err| err.into())
}
