use argparse;
use subprocess;

use super::errors;
use super::eval;
use super::model;


/// Return a subprocess::Exec instance from a command vector.
pub fn run<S>(cmd: &[S]) -> Result<(), errors::GardenError>
where S: AsRef<std::ffi::OsStr> {
    let mut exit_status: i32 = 1;

    if let Ok(mut p) = subprocess::Popen::create(
            cmd, subprocess::PopenConfig::default()) {
        exit_status = status(p.wait());
    }

    result_from_exit_status(exit_status)
}


/// Convert an exit status to Result<(), GardenError>.
pub fn result_from_exit_status(exit_status: i32) -> Result<(), errors::GardenError> {
    match exit_status {
        0 => Ok(()),
        _ => Err(errors::GardenError::ExitStatus(exit_status).into()),
    }
}


/// Extract the return status from subprocess::Result<subprocess::ExitStatus>.
pub fn status(result: subprocess::Result<subprocess::ExitStatus>) -> i32 {
    let mut exit_status: i32 = 1;

    if let Ok(status_result) = result {
        match status_result {
            subprocess::ExitStatus::Exited(status) => {
                exit_status = status as i32;
            }
            subprocess::ExitStatus::Signaled(status) => {
                exit_status = status as i32;
            }
            subprocess::ExitStatus::Other(status) => {
                exit_status = status;
            }
            _ => (),
        }
    }

    exit_status
}

/// Take a subprocess capture and return a string without trailing whitespace.
pub fn trim_stdout(capture: &subprocess::CaptureData) -> String {
    capture.stdout_str().trim_end().to_string()
}


/// Return a CaptureData result for a subprocess's stdout.
pub fn capture_stdout(exec: subprocess::Exec)
-> Result<subprocess::CaptureData, subprocess::PopenError> {
    exec.stdout(subprocess::Redirection::Pipe).capture()
}


/// Return a `subprocess::Exec` for a command.
pub fn exec_cmd<S>(command: &[S]) -> subprocess::Exec
    where S: AsRef<std::ffi::OsStr> {

    if command.len() > 1 {
        subprocess::Exec::cmd(&command[0]).args(&command[1..])
    } else {
        subprocess::Exec::cmd(&command[0])
    }
}

/// Return a `subprocess::Exec` that runs a command in the specified directory.
pub fn exec_in_dir<P, S>(command: &[S], path: P) -> subprocess::Exec
where P: AsRef<std::path::Path>, S: AsRef<std::ffi::OsStr> {
    exec_cmd(&command).cwd(path)
}

/// Run a command in the specified tree context.
/// Parameters:
/// - config: Mutable reference to a Configuration.
/// - context: Reference to the TreeContext to evaluate.
/// - quiet: Suppress messages when set true.
/// - verbose: increase verbosity of messages.
/// - command: String vector of the command to run.

pub fn exec_in_context<S>(
    config: &mut model::Configuration,
    context: &model::TreeContext,
    quiet: bool,
    verbose: bool,
    command: &[S],
) -> Result<(), errors::GardenError>
where S: AsRef<std::ffi::OsStr> {
    let path;
    // Immutable scope over tree
    {
        let tree = &config.trees[context.tree];
        path = tree.path.value.as_ref().unwrap().clone();

        // Sparse gardens/missing trees are ok -> skip these entries.
        if !model::print_tree(&tree, verbose, quiet) {
            return Ok(());
        }
    }
    // Evaluate the tree environment and run the command.
    let env = eval::environment(config, context);
    let command_vec = resolve_command(&command, &env);

    // Create an Exec object.
    let mut exec = exec_in_dir(&command_vec, &path);

    //  Update the command environment
    for (name, value) in &env {
        exec = exec.env(name, value);
    }

    result_from_exit_status(status(exec.join()))
}



/// The command might be a path that only exists inside the resolved
/// environment.  Resolve the path by looking for the presence of PATH
/// and updating cmdpath when it exists.

pub fn resolve_command<S>(
    command: &[S],
    env: &Vec<(String, String)>,
) -> Vec<String>
where S: AsRef<std::ffi::OsStr> {
    let mut cmdpath = std::path::PathBuf::from(&command[0]);
    if !cmdpath.is_absolute() {
        for (name, value) in env {
            if name == "PATH" {
                let fullpath = std::env::split_paths(&value).filter_map(|dir| {
                    let full_path = dir.join(&cmdpath);
                    if full_path.is_file() {
                        Some(full_path)
                    } else {
                        None
                    }
                }).next();

                if fullpath.is_some() {
                    cmdpath = fullpath.unwrap().to_path_buf();
                    break;
                }
            }
        }
    }

    // Create a copy of the command so where the first entry has been replaced
    // with a $PATH-resolved absolute path.
    let mut command_vec: Vec<String> = Vec::new();
    command_vec.reserve(command.len());

    command_vec.push(cmdpath.to_string_lossy().to_string());
    for arg in &command[1..] {
        let curpath = std::path::PathBuf::from(arg);
        command_vec.push(curpath.to_string_lossy().to_string());
    }

    command_vec
}

/// Split a vector into two vectors -- pre-dash and post-dash
pub fn split_on_dash<S>(
    strings: &[S],
    pre_dash: &mut Vec<String>,
    post_dash: &mut Vec<String>,
) where S: AsRef<std::ffi::OsStr> + std::string::ToString + std::cmp::PartialEq {

    let mut is_pre_dash = true;
    for string in strings {
        if is_pre_dash {
            if string.as_ref() == "--" {
                is_pre_dash = false;
                continue;
            }
            pre_dash.push((*string).to_string());
        } else {
            post_dash.push(string.to_string());
        }
    }
}


/// Return the current executable path.
pub fn current_exe() -> String {
    match std::env::current_exe() {
        Err(_) => "garden".to_string(),
        Ok(path) => path.to_string_lossy().to_string(),
    }
}


/// Parse arguments or exit with an error.
pub fn parse_args(parser: argparse::ArgumentParser, arguments: Vec<String>) {
    parser.parse(
        arguments, &mut std::io::stdout(), &mut std::io::stderr()
    ).map_err(|exit_status| std::process::exit(exit_status)).ok();
}
