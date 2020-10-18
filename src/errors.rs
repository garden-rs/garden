use thiserror::Error;


#[derive(Error, Debug)]
pub enum GardenError {
    #[error("assertion error: {0}")]
    AssertionError(String),

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

    #[error("file exists")]
    FileExists,

    #[error("file not found")]
    FileNotFound,

    #[error("unable to find '{garden}': No garden exists with that name")]
    GardenNotFound { garden: String },

    #[error("configuration IO error")]
    IOError,

    #[error("invalid configuration: {msg}")]
    InvalidConfiguration { msg: String },

    #[error("invalid argument: '{tree}' is not part of the '{garden}' garden")]
    InvalidGardenArgument { tree: String, garden: String },

    #[error("unable to read configuration: {err:?}")]
    ReadConfig { err: yaml_rust::ScanError },

    #[error("unable to read {path:?}: {err:?}")]
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

    #[error("invalid arguments")]
    Usage,

    #[error("unable to write configuration: {path:?}")]
    WriteConfigurationError { path: std::path::PathBuf },
}

impl std::convert::From<GardenError> for i32 {
    fn from(garden_err: GardenError) -> Self {
        // /usr/include/sysexits.h
        const EX_USAGE: i32 = 64;
        const EX_DATAERR: i32 = 65;
        const EX_SOFTWARE: i32 = 70;
        const EX_CANTCREAT: i32 = 73;
        const EX_IOERR: i32 = 74;
        const EX_CONFIG: i32 = 78;

        match garden_err {
            GardenError::AssertionError(_) => EX_SOFTWARE,
            GardenError::CreateConfigurationError { .. } => EX_CANTCREAT,
            GardenError::EmptyConfiguration { .. } => EX_CONFIG,
            GardenError::ExitStatus(status) => status,  // Explicit exit code
            GardenError::FileExists => EX_CANTCREAT,
            GardenError::FileNotFound => EX_IOERR,
            GardenError::GardenNotFound { .. } => EX_USAGE,
            GardenError::IOError => EX_IOERR,
            GardenError::InvalidConfiguration { .. } => EX_CONFIG,
            GardenError::InvalidGardenArgument { .. } => EX_USAGE,
            GardenError::ReadConfig { .. } => EX_DATAERR,
            GardenError::ReadFile { .. } => EX_IOERR,
            GardenError::SyncConfigurationError { .. } => EX_IOERR,
            GardenError::TreeNotFound { .. } => EX_USAGE,
            GardenError::Usage => EX_USAGE,
            GardenError::WriteConfigurationError { .. } => EX_CANTCREAT,
        }
    }
}
