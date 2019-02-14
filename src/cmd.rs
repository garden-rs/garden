extern crate subprocess;


pub fn run(cmd: &Vec<std::path::PathBuf>) -> i32 {
    let mut result: i32 = 1;

    if let Ok(mut p) = subprocess::Popen::create(
            cmd, subprocess::PopenConfig::default()) {

        if let Ok(exit_status) = p.wait() {
            match exit_status {
                subprocess::ExitStatus::Exited(status) => {
                    result = status as i32;
                }
                subprocess::ExitStatus::Signaled(status) => {
                    result = status as i32;
                }
                subprocess::ExitStatus::Other(status) => {
                    result = status;
                }
                _ => (),
            }
        }
    }
    return result;
}


pub fn status(result: subprocess::Result<subprocess::ExitStatus>) -> i32 {
    let mut exit_status: i32 = 1;

    if let Ok(status_result) = result {
        if let subprocess::ExitStatus::Exited(status) = status_result {
            exit_status = status as i32;
        }
    }

    return exit_status;
}
