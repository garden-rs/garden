use anyhow::Result;

use crate::{cli, cmd, constants, errors};

/// Main entry point for `garden gui <query> <command>...`.
pub fn main(options: &cli::MainOptions, arguments: &cli::Arguments) -> Result<()> {
    let capacity = get_capacity(options, arguments);
    let mut command = Vec::with_capacity(capacity);
    command.push(constants::GARDEN_GUI.as_ref());

    if let Some(config) = &options.config {
        command.push("--config".as_ref());
        command.push(config.as_os_str());
    }
    for debug in &options.debug {
        command.push("--debug".as_ref());
        command.push(debug.as_str().as_ref());
    }
    for define in &options.define {
        command.push("--define".as_ref());
        command.push(define.as_str().as_ref());
    }
    if let Some(root) = &options.root {
        command.push("--root".as_ref());
        command.push(root.as_os_str());
    }
    if options.quiet {
        command.push("--quiet".as_ref());
    }

    let verbose: String;
    if options.verbose > 0 {
        verbose = cli::verbose_string(options.verbose);
        command.push(verbose.as_str().as_ref());
    }

    for arg in &arguments.args {
        command.push(arg.as_str().as_ref());
    }

    let exec = cmd::exec_cmd(&command);
    let result = cmd::subprocess_result(exec.join());
    if result == Err(errors::EX_UNAVAILABLE) {
        eprintln!("error: garden-gui is not installed");
        eprintln!("error: run \"cargo install garden-gui\"");
    }

    errors::error_from_exit_status_result(result)
}

/// Calculate the size of the commands vector.
fn get_capacity(options: &cli::MainOptions, arguments: &cli::Arguments) -> usize {
    let mut capacity = 1; // garden-gui
    if options.config.is_some() {
        capacity += 2;
    }
    capacity += options.debug.len() * 2;
    capacity += options.define.len() * 2;
    if options.root.is_some() {
        capacity += 2;
    }
    if options.quiet {
        capacity += 1;
    }
    if options.verbose > 0 {
        capacity += 1;
    }
    capacity += arguments.args.len();

    capacity
}
