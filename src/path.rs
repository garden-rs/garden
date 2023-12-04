use super::errors;

/// Return the current directoy as a PathBuf.
pub fn current_dir() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
}

/// Return the current directory as a string.
pub fn current_dir_string() -> String {
    current_dir().to_string_lossy().to_string()
}

/// Return the home directory for the current user.
pub fn home_dir() -> std::path::PathBuf {
    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
}

/// Convert a Path into an absolute path.
pub fn abspath(path: &std::path::Path) -> std::path::PathBuf {
    path.to_path_buf()
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
}

/// Strip a prefix from a path.
pub fn strip_prefix(
    root: &std::path::Path,
    path: &std::path::Path,
) -> Result<std::path::PathBuf, errors::GardenError> {
    let stripped_path = if path.starts_with(root) {
        // Is the path a child of the current garden root?
        path.strip_prefix(root)
            .map_err(|err| {
                errors::GardenError::ConfigurationError(format!(
                    "{path:?} is not a child of {root:?}: {err:?}"
                ))
            })?
            .to_path_buf()
    } else {
        path.to_path_buf()
    };

    Ok(stripped_path)
}

/// Strip a prefix from a path. Returns a path as a string.
pub fn strip_prefix_into_string(
    root: &std::path::Path,
    path: &std::path::Path,
) -> Result<String, errors::GardenError> {
    let path_str = strip_prefix(root, path)?.to_string_lossy().to_string();
    Ok(path_str)
}
