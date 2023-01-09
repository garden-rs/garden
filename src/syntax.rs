/// Return true if the string contains 0-9 digits only
pub fn is_digit(string: &str) -> bool {
    string.chars().all(|c| c.is_ascii_digit())
}

/// Return true if `string` is an `$ exec` expression.
pub fn is_exec(string: &str) -> bool {
    string.starts_with("$ ")
}

/// Return true if `string` is a `:garden` expression.
pub fn is_garden(string: &str) -> bool {
    string.starts_with(':')
}

/// Return true if `string` is a `%group` expression.
pub fn is_group(string: &str) -> bool {
    string.starts_with('%')
}

/// Return true if `string` is a `@tree` expression.
pub fn is_tree(string: &str) -> bool {
    string.starts_with('@')
}

/// Return true if `string` is a `graft::value` expression.
pub fn is_graft(string: &str) -> bool {
    string.contains("::")
}

/// Return true if `string` ends in ".git". This is used to detect bare repositories.
pub fn is_git_dir(string: &str) -> bool {
    string.len() > 4 && string.ends_with(".git") && !string.ends_with("/.git")
}

/// Trim garden, group, and tree prefixes
pub fn trim(string: &str) -> &str {
    let needs_trim = is_group(string) || is_tree(string) || is_garden(string);
    if !string.is_empty() && needs_trim {
        &string[1..]
    } else {
        string
    }
}

/// Trim the "$ " prefix from an exec expression
pub fn trim_exec(string: &str) -> &str {
    let prefix = "$ ";
    let prefix_len = prefix.len();
    if string.len() >= prefix_len && string.starts_with(prefix) {
        &string[prefix_len..]
    } else {
        string
    }
}

/// Safely a string into pre and post-split references
pub fn split_string<'a>(string: &'a str, split: &str) -> (bool, &'a str, &'a str) {
    let end = string.len();
    let split_len = split.len();
    // split offset, everything up to this point is before the split
    let before = string.find(split).unwrap_or(end);
    let ok = before <= (end - split_len);
    // offset after the split
    let after = if ok { before + split_len } else { before };

    (ok, &string[..before], &string[after..])
}

/// Split a string into pre and post-graft namespace string refs
pub fn split_graft(string: &str) -> (bool, &str, &str) {
    split_string(string, "::")
}

/// Remove the graft basename leaving the remainder of the graft string.
pub fn trim_graft(string: &str) -> Option<String> {
    let (ok, _before, after) = split_graft(string);
    if !ok {
        return None;
    }

    let result;
    if is_garden(string) {
        result = ":".to_string() + after;
    } else if is_group(string) {
        result = "%".to_string() + after;
    } else if is_tree(string) {
        result = "@".to_string() + after;
    } else {
        result = after.to_string();
    }

    Some(result)
}

/// Return the graft basename.  "@foo::bar::baz" -> "foo"
pub fn graft_basename(string: &str) -> Option<String> {
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
pub fn escape_shell_variables(string: &str) -> String {
    let mut result = String::new();

    // Did we just see '$' ? If so, we might need to escape it.
    let mut potential_variable = false;
    for c in string.chars() {
        if potential_variable {
            if c.is_alphanumeric() || c == '_' {
                result.push('$'); // Escape $variable -> $$variable.
                result.push(c);
            } else if c == '$' {
                result.push('$'); // Escape $$ -> $$$$
                result.push('$');
                result.push('$');
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
