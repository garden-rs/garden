/// Construction functions for garden entities.
use super::cli;
use super::config;
use super::errors;
use super::model;

/// Construct an ApplicationContext from a path using default MainOptions
pub fn context_from_path(path: &str) -> Result<model::ApplicationContext, errors::GardenError> {
    let options = cli::MainOptions::new();
    let config = config::from_path_string(path, options.verbose)?;
    context_from_config(config, &options)
}

/// Construct an ApplicationContext from a Configuration and MainOptions
pub fn context_from_config(
    config: model::Configuration,
    options: &cli::MainOptions,
) -> Result<model::ApplicationContext, errors::GardenError> {
    let mut app = model::ApplicationContext::new(config, options.clone());
    config::read_grafts(&mut app)?;

    Ok(app)
}
