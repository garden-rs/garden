/// Macros
#[macro_use]
pub mod macros;

/// Command utilities
pub mod command;

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

/// Commands
pub mod cmds {
    use super::command;
    use super::config;
    use super::eval;
    use super::model;
    use super::query;

    /// Preset commands
    pub mod cmd;
    /// Exec command
    pub mod exec;
    /// Help command
    pub mod help;
    /// List command
    pub mod list;
}
