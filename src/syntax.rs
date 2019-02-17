/// Return true if `string` is an `$ exec` expression.
pub fn is_exec(string: &String) -> bool {
    string.starts_with("$ ")
}


/// Return true if `string` is a `:garden` expression.
pub fn is_garden(string: &String) -> bool {
    string.starts_with(":")
}


/// Return true if `string` is a `%group` expression.
pub fn is_group(string: &String) -> bool {
    string.starts_with("%")
}


/// Return true if `string` is a `@tree` expression.
pub fn is_tree(string: &String) -> bool {
    string.starts_with("@")
}


/// Trim garden, group, and tree prefixes
pub fn trim(string: &String) -> String {
    let mut value = string.to_string();
    value.remove(0);
    value
}


/// Trim the prefix from an exec expression
pub fn trim_exec(string: &String) -> String {
    let mut cmd = string.to_string();
    cmd.remove(0);
    cmd.remove(0);
    cmd
}
