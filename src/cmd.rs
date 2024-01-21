use crate::{display, errors, eval, model, syntax};

/// Convert an exit status to Result<(), GardenError>.
pub(crate) fn result_from_exit_status(exit_status: i32) -> Result<(), errors::GardenError> {
    match exit_status {
        errors::EX_OK => Ok(()),
        _ => Err(errors::GardenError::ExitStatus(exit_status)),
    }
}

/// Return an exit status code from a subprocess::Exec instance.
pub fn status(exec: subprocess::Exec) -> i32 {
    status_code(exec.join())
}

/// Return the status code from subprocess::Result<subprocess::ExitStatus>.
fn status_code(result: subprocess::Result<subprocess::ExitStatus>) -> i32 {
    match result {
        Ok(subprocess::ExitStatus::Exited(status)) => status as i32,
        Ok(subprocess::ExitStatus::Signaled(status)) => status as i32,
        Ok(subprocess::ExitStatus::Other(status)) => status,
        Ok(subprocess::ExitStatus::Undetermined) => errors::EX_ERROR,
        Err(_) => errors::EX_ERROR,
    }
}

/// Take a subprocess capture and return a string without trailing whitespace.
fn stdout(capture: &subprocess::CaptureData) -> String {
    capture.stdout_str().trim_end().to_string()
}

/// Convert a PopenError into a garden::errors::CommandError.
fn command_error_from_popen_error(
    command: String,
    popen_err: subprocess::PopenError,
) -> errors::CommandError {
    let status = match popen_err {
        subprocess::PopenError::IoError(err) => err.raw_os_error().unwrap_or(1),
        _ => 1,
    };
    errors::CommandError::ExitStatus { command, status }
}

/// Return a CaptureData result for a subprocess's stdout.
pub(crate) fn capture_stdout(
    exec: subprocess::Exec,
) -> Result<subprocess::CaptureData, errors::CommandError> {
    let command = exec.to_cmdline_lossy();
    exec.stdout(subprocess::Redirection::Pipe)
        .stderr(subprocess::NullFile {}) // Redirect stderr to /dev/null
        .capture()
        .map_err(|popen_err| command_error_from_popen_error(command, popen_err))
}

/// Return a trimmed stdout string for an subprocess::Exec instance.
pub fn stdout_to_string(exec: subprocess::Exec) -> Result<String, errors::CommandError> {
    Ok(stdout(&capture_stdout(exec)?))
}

/// Return a `subprocess::Exec` for a command.
pub(crate) fn exec_cmd<S>(command: &[S]) -> subprocess::Exec
where
    S: AsRef<std::ffi::OsStr>,
{
    if command.len() > 1 {
        subprocess::Exec::cmd(&command[0]).args(&command[1..])
    } else {
        subprocess::Exec::cmd(&command[0])
    }
}

/// Return a `subprocess::Exec` that runs a command in the specified directory.
pub fn exec_in_dir<P, S>(command: &[S], path: &P) -> subprocess::Exec
where
    P: AsRef<std::path::Path> + std::convert::AsRef<std::ffi::OsStr> + ?Sized,
    S: AsRef<std::ffi::OsStr>,
{
    exec_cmd(command).cwd(path).env("PWD", path)
}

/// Run a command in the specified tree context.
/// Parameters:
/// - config: Mutable reference to a Configuration.
/// - context: Reference to the TreeContext to evaluate.
/// - quiet: Suppress messages when set true.
/// - verbose: increase verbosity of messages.
/// - command: String vector of the command to run.

pub(crate) fn exec_in_context<S>(
    app_context: &model::ApplicationContext,
    config: &model::Configuration,
    context: &model::TreeContext,
    quiet: bool,
    verbose: u8,
    dry_run: bool,
    command: &[S],
) -> Result<(), errors::GardenError>
where
    S: AsRef<std::ffi::OsStr>,
{
    let path;
    if let Some(tree) = config.trees.get(&context.tree) {
        path = tree.path_as_ref()?;

        // Sparse gardens/missing trees are okay -> skip these entries.
        if !display::print_tree(tree, config.tree_branches, verbose, quiet) {
            return Ok(());
        }
    } else {
        return Ok(());
    }
    // Evaluate the tree environment and run the command.
    let env = eval::environment(app_context, config, context);
    let command_vec = resolve_command(command, &env);
    if verbose > 1 || dry_run {
        display::print_command_string_vec(&command_vec);
    }
    if dry_run {
        return Ok(());
    }

    // Create an Exec object.
    let mut exec = exec_in_dir(&command_vec, path);

    //  Update the command environment
    for (name, value) in &env {
        exec = exec.env(name, value);
    }

    result_from_exit_status(status(exec))
}

/// The command might be a path that only exists inside the resolved
/// environment.  Resolve the path by looking for the presence of PATH
/// and updating the command when it exists.

fn resolve_command<S>(command: &[S], env: &[(String, String)]) -> Vec<String>
where
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd_path = std::path::PathBuf::from(&command[0]);
    // Transform cmd_path into an absolute path.
    if !cmd_path.is_absolute() {
        for (name, value) in env {
            // Loop until we find PATH.
            if name == "PATH" {
                if let Some(path_buf) = std::env::split_paths(&value).find_map(|dir| {
                    let full_path = dir.join(&cmd_path);
                    if full_path.is_file() {
                        Some(full_path)
                    } else {
                        None
                    }
                }) {
                    cmd_path = path_buf;
                }
                // Once we've seen $PATH we're done.
                break;
            }
        }
    }

    // Create a copy of the command so where the first entry has been replaced
    // with a $PATH-resolved absolute path.
    let mut command_vec = Vec::with_capacity(command.len());
    command_vec.push(cmd_path.to_string_lossy().to_string());
    for arg in &command[1..] {
        let curpath = std::path::PathBuf::from(arg);
        command_vec.push(curpath.to_string_lossy().into());
    }

    command_vec
}

/// Return the current executable path.
pub(crate) fn current_exe() -> String {
    match std::env::current_exe() {
        Err(_) => "garden".into(),
        Ok(path) => path.to_string_lossy().into(),
    }
}

/// Given a command name, eg. "custom>", collect all of the custom command values
/// configured using the specified name. This function is used to gather
/// pre and post-commands associated with a command.
pub(crate) fn get_command_values(
    app_context: &model::ApplicationContext,
    context: &model::TreeContext,
    name: &str,
) -> Vec<String> {
    let config = match context.config {
        Some(config_id) => app_context.get_config(config_id),
        None => app_context.get_root_config(),
    };
    let mut vec_variables = Vec::new();

    // Global commands
    for (command_name, var) in &config.commands {
        if name == command_name {
            vec_variables.push(var.clone());
        }
    }

    // Tree commands
    if let Some(tree) = config.trees.get(&context.tree) {
        for (command_name, var) in &tree.commands {
            if name == command_name {
                vec_variables.push(var.clone());
            }
        }
    }

    // Optional garden command scope
    if let Some(garden_name) = &context.garden {
        if let Some(garden) = &config.gardens.get(garden_name) {
            for (command_name, var) in &garden.commands {
                if name == command_name {
                    vec_variables.push(var.clone());
                }
            }
        }
    }

    let mut commands = Vec::with_capacity(vec_variables.len() * 2);
    for variables in vec_variables.iter_mut() {
        let values = eval::variables_for_shell(app_context, config, variables, context);
        commands.extend(values);
    }

    commands
}

/// Recursively expand a command name to include its pre-commands and post-commands.
/// Self-referential loops are avoided. Duplicate commands are retained.
pub(crate) fn expand_command_names(
    app_context: &model::ApplicationContext,
    context: &model::TreeContext,
    name: &str,
) -> Vec<String> {
    let pre_name = syntax::pre_command(name);
    let post_name = syntax::post_command(name);
    let pre_commands = get_command_values(app_context, context, &pre_name);
    let post_commands = get_command_values(app_context, context, &post_name);

    let mut command_names = Vec::with_capacity(pre_commands.len() + 1 + post_commands.len());
    // Recursively expand pre-commands.
    for cmd_name in pre_commands.iter() {
        if cmd_name != name {
            // Avoid self-referential loops.
            command_names.extend(expand_command_names(app_context, context, cmd_name));
        }
    }
    command_names.push(name.to_string());
    // Recursively expand post-commands.
    for cmd_name in post_commands.iter() {
        if cmd_name != name {
            // Avoid self-referential loops.
            command_names.extend(expand_command_names(app_context, context, cmd_name));
        }
    }

    command_names
}
