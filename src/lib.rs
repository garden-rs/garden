/// Macros
#[macro_use]
pub mod macros;

/// Command utilities
pub mod cmd;

/// Commands
pub mod cmds;

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
