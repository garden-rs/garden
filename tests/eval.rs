#[macro_use]
extern crate garden;
extern crate dirs;

mod common;

#[test]
fn tree_variable() {
    let mut config = common::garden_config();
    let tree_idx: garden::model::TreeIndex = 0;
    let result = garden::eval::value(&mut config, "${prefix}", tree_idx, None);

    let path_buf = dirs::home_dir().unwrap();
    let home_dir= path_buf.to_string_lossy();

    assert!(result.starts_with(home_dir.as_ref()));
    assert!(result.ends_with("/.local"));
}
