extern crate subprocess;

use ::eval;
use ::model;


/// Return a subprocess::Exec instance from a command vector.
pub fn run<S>(cmd: &[S]) -> i32
where S: AsRef<std::ffi::OsStr> {
    let mut exit_status: i32 = 1;

    if let Ok(mut p) = subprocess::Popen::create(
            cmd, subprocess::PopenConfig::default()) {
        exit_status = status(p.wait());
    }

    exit_status
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

    subprocess::Exec::cmd(&command[0]).args(&command[1..])
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
) -> i32
where S: AsRef<std::ffi::OsStr> {
    let path;
    // Immutable scope over tree
    {
        let tree = &config.trees[context.tree];
        path = tree.path.value.as_ref().unwrap().clone();

        // Sparse gardens/missing trees are ok -> skip these entries.
        if !model::print_tree(&tree, verbose, quiet) {
            return 0;
        }
    }

    // Evaluate the tree environment and run the command.
    let env = eval::environment(config, context);
    let mut exec = exec_in_dir(command, &path);
    //  Update the command environment
    for (name, value) in &env {
        exec = exec.env(name, value);
    }

    status(exec.join())
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

/// Return codes from programs.  Cf. /usr/include/sysexits.h
pub enum ExitCode {
    Success,
    Config,
    FileExists,
    FileNotFound,
    IOError,
    Usage,
}


impl std::convert::From<ExitCode> for i32 {
    fn from(exit_code: ExitCode) -> Self {
        match exit_code {
            ExitCode::Config => 78,  // EX_CONFIG,
            ExitCode::FileExists => 64, // EX_USAGE
            ExitCode::FileNotFound => 74,  // EX_IOERR
            ExitCode::IOError => 74,  // EX_IOERR
            ExitCode::Success => 0,  // EXIT_SUCCESS
            ExitCode::Usage => 64,  // EX_USAGE
        }
    }
}
