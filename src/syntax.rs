/// Return true if `string` is a `@tree` expression.
pub fn is_tree(string: &String) -> bool {
    string.starts_with("@")
}


/// Return true if `string` is a `%group` expression.
pub fn is_group(string: &String) -> bool {
    string.starts_with("%")
}


/// Return true if `string` is a `:garden` expression.
pub fn is_garden(string: &String) -> bool {
    string.starts_with(":")
}
