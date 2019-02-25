extern crate subprocess;

use ::eval;
use ::model;


pub fn run(cmd: &Vec<std::path::PathBuf>) -> i32 {
    let mut exit_status: i32 = 1;

    if let Ok(mut p) = subprocess::Popen::create(
            cmd, subprocess::PopenConfig::default()) {
        exit_status = status(p.wait());
    }

    exit_status
}


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
pub fn capture_stdout(mut exec: subprocess::Exec)
-> Result<subprocess::CaptureData, subprocess::PopenError> {
    exec.stdout(subprocess::Redirection::Pipe).capture()
}

/// Run a command in the specified tree context.
/// Parameters:
/// - config: Mutable reference to a Configuration.
/// - context: Reference to the TreeContext to evaluate.
/// - quiet: Suppress messages when set true.
/// - verbose: increase verbosity of messages.
/// - command: String vector of the command to run.

pub fn exec_in_context(
    config: &mut model::Configuration,
    context: &model::TreeContext,
    quiet: bool,
    verbose: bool,
    command: &Vec<String>,
) -> i32 {
    // Evaluate the tree environment and run the command.
    let env = eval::environment(config, context);
    // Exec each command in the tree's context
    let tree = &config.trees[context.tree];
    let path = tree.path.value.as_ref().unwrap();
    // Sparse gardens/missing trees are ok -> skip these entries.
    if !std::path::PathBuf::from(&path).exists() {
        if !quiet {
            if verbose {
                eprintln!("# {}: {} (skipped)", tree.name, path);
            } else {
                eprintln!("# {} (skipped)", tree.name);
            }
        }
        return 0;  // Missing trees are ok
    }
    if !quiet {
        if verbose {
            eprintln!("# {}: {}", tree.name, path);
        } else {
            eprintln!("# {}", tree.name);
        }
    }
    let mut exec = subprocess::Exec::cmd(&command[0])
        .args(&command[1..])
        .cwd(&path);

    // Update the command environment
    for (name, value) in &env {
        exec = exec.env(name, value);
    }

    status(exec.join())
}
