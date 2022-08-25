/// Construction functions for garden entities.
use super::config;
use super::errors;
use super::model;

pub fn context_from_path(
    path: &str,
    options: model::CommandOptions,
) -> Result<model::ApplicationContext, errors::GardenError> {
    let config = config::from_path_string(path, options.verbose)?;
    context_from_config(config, options)
}

pub fn context_from_config(
    config: model::Configuration,
    options: model::CommandOptions,
) -> Result<model::ApplicationContext, errors::GardenError> {
    let mut app = model::ApplicationContext::new(config, options);
    config::read_grafts(&mut app)?;

    Ok(app)
}
