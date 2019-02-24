/// Macros
#[macro_use]
pub mod macros;

/// Command utilities
pub mod cmd;

/// Configuration
pub mod config;

/// Variable evaluation
pub mod eval;

/// Model objects
pub mod model;

/// Queries, configuration lookups
pub mod query;

/// Command-line syntax conventions
pub mod syntax;

///
/// Private modules
///

/// YAML/JSON reader
mod config_yaml;

///
/// Commands
///

pub mod cmds {
    /// Preset commands
    pub mod cmd;
    /// Exec command
    pub mod exec;
    /// Eval command
    pub mod eval;
    /// Help command
    pub mod help;
    /// List command
    pub mod list;
    /// Shell command
    pub mod shell;
}
