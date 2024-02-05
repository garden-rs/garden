/// Return true if the string contains 0-9 digits only
#[inline]
pub(crate) fn is_digit(string: &str) -> bool {
    string.chars().all(|c| c.is_ascii_digit())
}

/// Return true if `string` is an `$ exec` expression.
#[inline]
pub(crate) fn is_exec(string: &str) -> bool {
    string.starts_with("$ ")
}

/// Return true if `string` is a `:garden` expression.
#[inline]
pub(crate) fn is_garden(string: &str) -> bool {
    string.starts_with(':')
}

/// Return true if `string` is a `%group` expression.
#[inline]
pub(crate) fn is_group(string: &str) -> bool {
    string.starts_with('%')
}

/// Return true if `string` is a `@tree` expression.
#[inline]
pub(crate) fn is_tree(string: &str) -> bool {
    string.starts_with('@')
}

/// Return true if `string` is a variable "replace" operation.
#[inline]
pub(crate) fn is_append_op(string: &str) -> bool {
    string.ends_with('+')
}

/// Return true if `string` is a variable "append" operation.
#[inline]
pub(crate) fn is_replace_op(string: &str) -> bool {
    string.ends_with('=')
}

/// Return true if `string` is a `graft::value` expression.
#[inline]
pub(crate) fn is_graft(string: &str) -> bool {
    string.contains("::")
}

/// Return true if `string` is a candidate for evaluation.
/// Returns true for strings with ${vars}  and "$ exec" expressions.
#[inline]
pub(crate) fn is_eval_candidate(string: &str) -> bool {
    string.contains('$')
}

/// Return true if `string` ends in ".git". This is used to detect bare repositories.
#[inline]
pub(crate) fn is_git_dir(string: &str) -> bool {
    string.len() > 4 && string.ends_with(".git") && !string.ends_with("/.git")
}

/// Trim garden, group, and tree prefixes
#[inline]
pub(crate) fn trim(string: &str) -> &str {
    let needs_trim = is_group(string) || is_tree(string) || is_garden(string);
    if !string.is_empty() && needs_trim {
        &string[1..]
    } else {
        string
    }
}

/// Trim the "$ " prefix from an exec expression
#[inline]
pub(crate) fn trim_exec(string: &str) -> &str {
    let prefix = "$ ";
    let prefix_len = prefix.len();
    if string.len() >= prefix_len && string.starts_with(prefix) {
        &string[prefix_len..]
    } else {
        string
    }
}

/// Trim "+" and "=" suffixes in-place.
#[inline]
pub(crate) fn trim_op_inplace(string: &mut String) {
    let len = string.len();
    if len > 1 {
        string.remove(len - 1);
    }
}

/// Safely a string into pre and post-split references
#[inline]
pub(crate) fn split_string<'a>(string: &'a str, split: &str) -> (bool, &'a str, &'a str) {
    let end = string.len();
    let split_len = split.len();
    if end < split_len {
        return (false, string, "");
    }
    // split offset, everything up to this point is before the split
    let before = string.find(split).unwrap_or(end);
    let ok = before <= (end - split_len);
    // offset after the split
    let after = if ok { before + split_len } else { before };

    (ok, &string[..before], &string[after..])
}

/// Split a string into pre and post-graft namespace string refs
#[inline]
pub(crate) fn split_graft(string: &str) -> (bool, &str, &str) {
    split_string(string, "::")
}

/// Remove the graft basename leaving the remainder of the graft string.
#[inline]
pub(crate) fn trim_graft(string: &str) -> Option<String> {
    let (ok, _before, after) = split_graft(string);
    if !ok {
        return None;
    }

    let result;
    if is_garden(string) {
        result = string!(":") + after;
    } else if is_group(string) {
        result = string!("%") + after;
    } else if is_tree(string) {
        result = string!("@") + after;
    } else {
        result = after.to_string();
    }

    Some(result)
}

/// Return the graft basename.  "@foo::bar::baz" -> "foo"
#[inline]
pub(crate) fn graft_basename(string: &str) -> Option<String> {
    let (ok, before, _after) = split_graft(string);
    if !ok {
        return None;
    }

    let result = if is_garden(string) || is_group(string) || is_tree(string) {
        trim(before)
    } else {
        before
    }
    .to_string();

    Some(result)
}

/// Escape $variable into $$variable for evaluation by shellexpand.
#[inline]
pub(crate) fn escape_shell_variables(string: &str) -> String {
    let mut result = String::new();

    // Did we just see '$' ? If so, we might need to escape it.
    let mut potential_variable = false;
    for c in string.chars() {
        if potential_variable {
            if c.is_alphanumeric() || c == '_' {
                result.push('$'); // Escape $variable -> $$variable.
                result.push(c);
            } else if c == '$' {
                result.push('$'); // Escape $$ -> $
            } else {
                result.push(c);
            }
            potential_variable = false;
        } else {
            // Push the value into the stream.
            result.push(c);

            // If the current value is '$' then the next loop may need to escape it.
            potential_variable = c == '$';
        }
    }

    result
}

/// Return the value of a boolean as a string.
#[inline]
pub(crate) fn bool_to_string(value: bool) -> String {
    match value {
        true => string!("true"),
        false => string!("false"),
    }
}

/// Return the value of a string as a boolean. Accepts "true", "false", "1" and "0".
#[inline]
pub(crate) fn string_to_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

/// Add a pre-command suffix to a command name.
#[inline]
pub(crate) fn pre_command(name: &str) -> String {
    format!("{}<", name)
}

/// Add a post-command suffix to a command name.
#[inline]
pub(crate) fn post_command(name: &str) -> String {
    format!("{}>", name)
}

/// Unit tests
#[cfg(test)]
mod tests {
    #[test]
    fn is_garden() {
        assert!(super::is_garden(":garden"), ":garden is a garden");
        assert!(!super::is_garden("garden"), "garden is not a garden");
    }

    #[test]
    fn is_graft() {
        assert!(super::is_graft("foo::bar"), "foo::bar is a graft");
        assert!(!super::is_graft("foo"), "foo is not a graft");
    }

    #[test]
    fn is_group() {
        assert!(super::is_group("%group"), "%group is a group");
        assert!(!super::is_group("group"), "group is not a group");
    }

    #[test]
    fn is_tree() {
        assert!(super::is_tree("@tree"), "@tree is a tree");
        assert!(!super::is_tree("tree"), "tree is not a tree");
    }

    #[test]
    fn is_git_dir() {
        assert!(super::is_git_dir("tree.git"), "tree.git is a git dir");
        assert!(
            super::is_git_dir("/src/tree.git"),
            "/src/tree.git is a git dir"
        );
        assert!(
            !super::is_git_dir("src/tree/.git"),
            "src/tree/.git is a git dir"
        );
        assert!(!super::is_git_dir(".git"), ".git is a git dir");
        assert!(!super::is_git_dir("/.git"), "/.git is a git dir");
    }

    #[test]
    fn split_string_ok() {
        let (ok, pre, post) = super::split_string("foo::bar", "::");
        assert!(ok, "split :: on foo::bar is ok");
        assert_eq!(pre, "foo");
        assert_eq!(post, "bar");
    }

    #[test]
    fn split_string_empty() {
        let (ok, pre, post) = super::split_string("foo::", "::");
        assert!(ok, "split :: on foo:: is ok");
        assert_eq!(pre, "foo");
        assert_eq!(post, "");
    }

    #[test]
    fn split_string_not_found() {
        let (ok, pre, post) = super::split_string("foo", "::");
        assert!(!ok, "split :: on foo is false");
        assert_eq!(pre, "foo");
        assert_eq!(post, "");
    }

    #[test]
    fn split_graft_ok() {
        let (ok, pre, post) = super::split_graft("foo::bar");
        assert!(ok, "split_graft on foo::bar is ok");
        assert_eq!(pre, "foo");
        assert_eq!(post, "bar");
    }

    #[test]
    fn split_graft_nested_ok() {
        let (ok, pre, post) = super::split_graft("@foo::bar::baz");
        assert!(ok, "split_graft on @foo::bar::baz is ok");
        assert_eq!(pre, "@foo");
        assert_eq!(post, "bar::baz");
    }

    #[test]
    fn split_graft_empty() {
        let (ok, pre, post) = super::split_graft("foo::");
        assert!(ok, "split_graft on foo:: is ok");
        assert_eq!(pre, "foo");
        assert_eq!(post, "");
    }

    #[test]
    fn split_graft_not_found() {
        let (ok, pre, post) = super::split_graft("foo");
        assert!(!ok, "split_graft on foo is false");
        assert_eq!(pre, "foo");
        assert_eq!(post, "");
    }

    #[test]
    fn trim_exec() {
        assert_eq!("cmd", super::trim_exec("$ cmd"));
        assert_eq!("$cmd", super::trim_exec("$cmd"));
        assert_eq!("cmd", super::trim_exec("cmd"));
        assert_eq!("", super::trim_exec("$ "));
        assert_eq!("$", super::trim_exec("$"));
        assert_eq!("", super::trim_exec(""));
    }

    #[test]
    fn trim_graft() {
        let value = super::trim_graft("foo::bar::baz");
        assert!(value.is_some());
        assert_eq!("bar::baz", value.unwrap());

        let value = super::trim_graft("@foo::bar::baz");
        assert!(value.is_some());
        assert_eq!("@bar::baz", value.unwrap());

        let value = super::trim_graft("%foo::bar::baz");
        assert!(value.is_some());
        assert_eq!("%bar::baz", value.unwrap());

        let value = super::trim_graft(":foo::bar::baz");
        assert!(value.is_some());
        assert_eq!(":bar::baz", value.unwrap());

        let value = super::trim_graft("foo::bar");
        assert!(value.is_some());
        assert_eq!("bar", value.unwrap());

        let value = super::trim_graft("foo");
        assert!(value.is_none());
    }

    #[test]
    fn graft_basename() {
        let value = super::graft_basename("foo");
        assert!(value.is_none());

        let value = super::graft_basename(":foo");
        assert!(value.is_none());

        let value = super::graft_basename("%foo");
        assert!(value.is_none());

        let value = super::graft_basename("@foo");
        assert!(value.is_none());

        let value = super::graft_basename("foo::bar");
        assert!(value.is_some());
        assert_eq!("foo", value.unwrap());

        let value = super::graft_basename(":foo::bar");
        assert!(value.is_some());
        assert_eq!("foo", value.unwrap());

        let value = super::graft_basename("%foo::bar");
        assert!(value.is_some());
        assert_eq!("foo", value.unwrap());

        let value = super::graft_basename("@foo::bar");
        assert!(value.is_some());
        assert_eq!("foo", value.unwrap());

        let value = super::graft_basename("foo::bar::baz");
        assert!(value.is_some());
        assert_eq!("foo", value.unwrap());

        let value = super::graft_basename(":foo::bar::baz");
        assert!(value.is_some());
        assert_eq!("foo", value.unwrap());

        let value = super::graft_basename("%foo::bar::baz");
        assert!(value.is_some());
        assert_eq!("foo", value.unwrap());

        let value = super::graft_basename("@foo::bar::baz");
        assert!(value.is_some());
        assert_eq!("foo", value.unwrap());
    }

    #[test]
    fn escape_shell_variables() {
        let value = super::escape_shell_variables("$");
        assert_eq!(value, "$");

        let value = super::escape_shell_variables("$ ");
        assert_eq!(value, "$ ");

        let value = super::escape_shell_variables("$$");
        assert_eq!(value, "$$");

        let value = super::escape_shell_variables("$_");
        assert_eq!(value, "$$_");

        let value = super::escape_shell_variables("$a");
        assert_eq!(value, "$$a");

        let value = super::escape_shell_variables("$_a");
        assert_eq!(value, "$$_a");

        let value = super::escape_shell_variables("$ echo");
        assert_eq!(value, "$ echo");

        let value = super::escape_shell_variables("embedded $$ value");
        assert_eq!(value, "embedded $$ value");

        let value = super::escape_shell_variables("$variable");
        assert_eq!(value, "$$variable");

        let value = super::escape_shell_variables("$$variable");
        assert_eq!(value, "$$variable");

        let value = super::escape_shell_variables("${braces}${ignored}");
        assert_eq!(value, "${braces}${ignored}");

        let value = super::escape_shell_variables("$a ${b} $c $");
        assert_eq!(value, "$$a ${b} $$c $");

        // Escaped ${braced} value
        let value = super::escape_shell_variables("echo $${value[@]:0:1}");
        assert_eq!(value, "echo $${value[@]:0:1}");
    }
}
