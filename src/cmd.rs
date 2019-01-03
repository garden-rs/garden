pub fn error(args: std::fmt::Arguments) {
    eprintln!("error: {}", args);
    std::process::exit(1);
}

pub fn debug(args: std::fmt::Arguments) {
    eprintln!("debug: {}", args);
}

pub fn get_status(cmd: &Vec<std::path::PathBuf>) -> i32 {
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
