use crate::{constants, errors};

/// Return the current directory as a PathBuf.
pub fn current_dir() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(constants::DOT))
}

/// Return the current directory as a string.
pub(crate) fn current_dir_string() -> String {
    current_dir().to_string_lossy().to_string()
}

/// Return the home directory for the current user.
pub(crate) fn home_dir() -> std::path::PathBuf {
    dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
}

/// Canonicalize a path while avoiding problematic UNC paths on Windows.
pub(crate) fn canonicalize<P: AsRef<std::path::Path>>(
    path: P,
) -> std::io::Result<std::path::PathBuf> {
    dunce::canonicalize(path)
}

/// Convert a Path into an absolute path. Return the original path if it cannot be canonicalized.
pub fn abspath<P: AsRef<std::path::Path> + std::marker::Copy>(path: P) -> std::path::PathBuf {
    canonicalize(path).unwrap_or(path.as_ref().to_path_buf())
}

/// Return the basename of a path-like string.
pub(crate) fn str_basename(path: &str) -> &str {
    let basename = if path.contains('/') {
        path.split('/').next_back().unwrap_or(path)
    } else if path.contains('\\') {
        path.split('\\').next_back().unwrap_or(path)
    } else {
        path
    };

    basename
}

/// Return true if the basename is a known shell.
pub(crate) fn is_shell(basename: &str) -> bool {
    matches!(
        basename,
        constants::SHELL_BASH
            | constants::SHELL_DASH
            | constants::SHELL_KSH
            | constants::SHELL_SH
            | constants::SHELL_ZSH
    )
}

/// Strip a prefix from a path.
pub(crate) fn strip_prefix(
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
pub(crate) fn strip_prefix_into_string(
    root: &std::path::Path,
    path: &std::path::Path,
) -> Result<String, errors::GardenError> {
    let path_str = strip_prefix(root, path)?.to_string_lossy().to_string();
    Ok(path_str)
}
