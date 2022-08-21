use thiserror::Error;

#[derive(Error, Debug)]
pub enum GardenError {
    #[error("assertion error: {0}")]
    AssertionError(String),

    #[error("configuration error: {0}")]
    ConfigurationError(String),

    #[error("{path:?}: unable to create configuration: {err}")]
    CreateConfigurationError {
        path: std::path::PathBuf,
        err: std::io::Error,
    },

    #[error("invalid configuration: empty document: {path:?}")]
    EmptyConfiguration { path: std::path::PathBuf },

    /// ExitStatus is used to exit without printing an error message.
    #[error("exit status {0}")]
    ExitStatus(i32),

    #[error("{0}")]
    FileExists(String),

    #[error("file not found")]
    FileNotFound,

    #[error("unable to find '{garden}': No garden exists with that name")]
    GardenNotFound { garden: String },

    #[error("'{garden}' is not a valid garden glob pattern")]
    GardenPatternError { garden: String },

    #[error("{0}")]
    IOError(String),

    #[error("invalid configuration: {msg}")]
    InvalidConfiguration { msg: String },

    #[error("invalid argument: '{tree}' is not part of the '{garden}' garden")]
    InvalidGardenArgument { tree: String, garden: String },

    #[error("{0}")]
    OSError(String),

    #[error("unable to read {path:?}\nerror: {err}")]
    ReadConfig {
        err: yaml_rust::ScanError,
        path: String,
    },

    #[error("unable to read {path:?}: {err}")]
    ReadFile {
        path: std::path::PathBuf,
        err: std::io::Error,
    },

    #[error("unable to sync configuration: {path:?}: {err:?}")]
    SyncConfigurationError {
        path: std::path::PathBuf,
        err: std::io::Error,
    },

    #[error("unable to find '{tree}': No tree exists with that name")]
    TreeNotFound { tree: String },

    #[error("invalid arguments: {0}")]
    Usage(String),

    #[error("error creating {tree:?}: 'git checkout' returned exit status {status:?}")]
    WorktreeGitCheckoutError { tree: String, status: i32 },

    #[error("unable to find worktree {worktree:?} for {tree:?}")]
    WorktreeNotFound { worktree: String, tree: String },

    #[error("error creating worktree parent '{worktree:?}' for '{tree:?}'")]
    WorktreeParentCreationError { worktree: String, tree: String },

    #[error("unable to plant {tree:?}: worktree parent {parent:?} has not been planted")]
    WorktreeParentNotPlantedError { parent: String, tree: String },

    #[error("unable to write configuration: {path:?}")]
    WriteConfigurationError { path: std::path::PathBuf },
}

#[derive(Error, Debug)]
pub enum CommandError {
    /// ExitStatus is used to exit without printing an error message.
    #[error("{command} returned exit status {status}")]
    ExitStatus { command: String, status: i32 },
}

// /usr/include/sysexits.h
pub const EX_OK: i32 = 0;
pub const EX_USAGE: i32 = 64;
pub const EX_DATAERR: i32 = 65;
pub const EX_SOFTWARE: i32 = 70;
pub const EX_OSERR: i32 = 71;
pub const EX_CANTCREAT: i32 = 73;
pub const EX_IOERR: i32 = 74;
pub const EX_CONFIG: i32 = 78;

impl std::convert::From<GardenError> for i32 {
    fn from(garden_err: GardenError) -> Self {
        match garden_err {
            GardenError::AssertionError(_) => EX_SOFTWARE,
            GardenError::ConfigurationError(_) => EX_CONFIG,
            GardenError::CreateConfigurationError { .. } => EX_CANTCREAT,
            GardenError::EmptyConfiguration { .. } => EX_CONFIG,
            GardenError::ExitStatus(status) => status, // Explicit exit code
            GardenError::FileExists(_) => EX_CANTCREAT,
            GardenError::FileNotFound => EX_IOERR,
            GardenError::GardenNotFound { .. } => EX_USAGE,
            GardenError::GardenPatternError { .. } => EX_DATAERR,
            GardenError::IOError(_) => EX_IOERR,
            GardenError::InvalidConfiguration { .. } => EX_CONFIG,
            GardenError::InvalidGardenArgument { .. } => EX_USAGE,
            GardenError::OSError(_) => EX_OSERR,
            GardenError::ReadConfig { .. } => EX_DATAERR,
            GardenError::ReadFile { .. } => EX_IOERR,
            GardenError::SyncConfigurationError { .. } => EX_IOERR,
            GardenError::TreeNotFound { .. } => EX_USAGE,
            GardenError::Usage(_) => EX_USAGE,
            GardenError::WorktreeGitCheckoutError { .. } => EX_CANTCREAT,
            GardenError::WorktreeParentCreationError { .. } => EX_CANTCREAT,
            GardenError::WorktreeParentNotPlantedError { .. } => EX_CONFIG,
            GardenError::WorktreeNotFound { .. } => EX_CONFIG,
            GardenError::WriteConfigurationError { .. } => EX_CANTCREAT,
        }
    }
}
