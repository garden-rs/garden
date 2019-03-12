extern crate subprocess;

#[cfg(test)]
mod integration {

#[test]
fn garden_init() {
    let cmd = [
        "./target/debug/garden",
        "--chdir", "./tests/init",
        "init", "example/vx",
    ];
    let exec = garden::cmd::exec_in_dir(&cmd, ".");
    let exit_status = garden::cmd::status(exec.join());
    assert_eq!(exit_status, 0);

    assert!(std::path::PathBuf::from("./tests/init/example").exists());
    assert!(std::path::PathBuf::from("./tests/init/example/vx").exists());
    assert!(std::path::PathBuf::from("./tests/init/example/vx/repo").exists());
    assert!(std::path::PathBuf::from("./tests/init/example/vx/repo/.git").exists());
}

}  // integration
