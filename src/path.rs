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
