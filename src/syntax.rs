/// Return true if `string` is an `$ exec` expression.
pub fn is_exec(string: &str) -> bool {
    string.starts_with("$ ")
}


/// Return true if `string` is a `:garden` expression.
pub fn is_garden(string: &str) -> bool {
    string.starts_with(":")
}


/// Return true if `string` is a `%group` expression.
pub fn is_group(string: &str) -> bool {
    string.starts_with("%")
}


/// Return true if `string` is a `@tree` expression.
pub fn is_tree(string: &str) -> bool {
    string.starts_with("@")
}


/// Trim garden, group, and tree prefixes
pub fn trim(string: &str) -> String {
    let mut value = string.to_string();
    value.remove(0);
    value
}


/// Trim the prefix from an exec expression
pub fn trim_exec(string: &str) -> String {
    let mut cmd = string.to_string();
    cmd.remove(0);
    cmd.remove(0);
    cmd
}
