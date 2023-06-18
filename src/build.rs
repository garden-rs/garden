/// Construction functions for garden entities.
use super::cli;
use super::errors;
use super::model;

/// Construct an ApplicationContext from a path using default MainOptions
pub fn context_from_path(path: &str) -> Result<model::ApplicationContext, errors::GardenError> {
    model::ApplicationContext::from_path(path)
}

/// Construct an ApplicationContext from a Configuration and MainOptions
pub fn context_from_config(
    config: model::Configuration,
    options: &cli::MainOptions,
) -> Result<model::ApplicationContext, errors::GardenError> {
    model::ApplicationContext::new(config, options.clone())
}
