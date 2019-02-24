extern crate subprocess;


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
