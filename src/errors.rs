use thiserror::Error;


#[derive(Error, Debug)]
pub enum GardenError {

    #[error("{path:?}: unable to create configuration: {err}")]
    CreateConfigurationError {
        path: std::path::PathBuf,
        err: std::io::Error,
    },

    #[error("invalid configuration: empty document: {path:?}")]
    EmptyConfiguration {
        path: std::path::PathBuf,
    },

    #[error("file exists")]
    FileExists,

    #[error("file not found")]
    FileNotFound,

    #[error("unable to find '{garden}': No garden exists with that name")]
    GardenNotFound {
        garden: String,
    },

    #[error("configuration IO error")]
    IOError,

    #[error("invalid configuration: {msg}")]
    InvalidConfiguration {
        msg: String,
    },

    #[error("invalid argument: '{tree}' is not part of the '{garden}' garden")]
    InvalidGardenArgument {
        tree: String,
        garden: String,
    },

    #[error("unable to read configuration: {err:?}")]
    ReadConfig {
        err: yaml_rust::ScanError,
    },

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
    WriteConfigurationError {
        path: std::path::PathBuf,
    },
}

impl std::convert::From<GardenError> for i32 {
    fn from(garden_err: GardenError) -> Self {
        match garden_err {
            GardenError::CreateConfigurationError { .. } => 78,  // EX_CONFIG
            GardenError::EmptyConfiguration { .. } => 78,  // EX_CONFIG
            GardenError::FileExists => 64, // EX_USAGE
            GardenError::FileNotFound => 74,  // EX_IOERR
            GardenError::GardenNotFound { .. } => 78,  // EX_USAGE
            GardenError::IOError => 74,  // EX_IOERR
            GardenError::InvalidConfiguration { .. } => 78,  // EX_CONFIG
            GardenError::InvalidGardenArgument { .. } => 78,  // EX_USAGE
            GardenError::ReadConfig { .. } => 74,  // EX_IOERR
            GardenError::ReadFile { .. } => 74,  // EX_IOERR
            GardenError::SyncConfigurationError { .. } => 74,  // EX_IOERR
            GardenError::TreeNotFound { .. } => 78,  // EX_USAGE
            GardenError::Usage => 64,  // EX_USAGE
            GardenError::WriteConfigurationError { .. } => 74,  // EX_IOERR
        }
    }
}
