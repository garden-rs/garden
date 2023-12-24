/// YAML reader
pub mod reader;

/// YAML writer
pub mod writer;

use crate::{errors, model, path};

/// Search for configuration in the following locations:
///  .
///  ./garden
///  ./etc/garden
///  ~/.config/garden
///  ~/etc/garden
///  /etc/garden

pub fn search_path() -> Vec<std::path::PathBuf> {
    // Result: Vec<PathBufs> in priority order
    let mut paths: Vec<std::path::PathBuf> = Vec::new();

    let current_dir = path::current_dir();
    let home_dir = path::home_dir();

    // . Current directory
    paths.push(current_dir.clone());

    // ./garden
    let mut current_garden_dir = current_dir.clone();
    current_garden_dir.push("garden");
    if current_garden_dir.exists() {
        paths.push(current_garden_dir);
    }

    // ./etc/garden
    let mut current_etc_garden_dir = current_dir;
    current_etc_garden_dir.push("etc");
    current_etc_garden_dir.push("garden");
    if current_etc_garden_dir.exists() {
        paths.push(current_etc_garden_dir);
    }

    // $XDG_CONFIG_HOME/garden (typically ~/.config/garden)
    paths.push(xdg_dir());

    // ~/etc/garden
    let mut home_etc_dir = home_dir;
    home_etc_dir.push("etc");
    home_etc_dir.push("garden");
    if home_etc_dir.exists() {
        paths.push(home_etc_dir);
    }

    // /etc/garden
    let etc_garden = std::path::PathBuf::from("/etc/garden");
    if etc_garden.exists() {
        paths.push(etc_garden);
    }

    paths
}

/// $XDG_CONFIG_HOME/garden (typically ~/.config/garden)
pub fn xdg_dir() -> std::path::PathBuf {
    let mut home_config_dir;

    if let Ok(xdg_dirs) = xdg::BaseDirectories::new() {
        home_config_dir = xdg_dirs.get_config_home();
    } else {
        home_config_dir = path::home_dir();
        home_config_dir.push(".config")
    }
    home_config_dir.push("garden");

    home_config_dir
}

/// Parse and apply configuration from a YAML/JSON string
pub fn parse(
    app_context: &model::ApplicationContext,
    config_string: &str,
    verbose: u8,
    cfg: &mut model::Configuration,
) -> Result<(), errors::GardenError> {
    reader::parse(app_context, config_string, verbose, cfg)?;
    // Initialize the configuration now that the values have been read.
    cfg.initialize(app_context);

    Ok(())
}

/// Read grafts into the root configuration on down.
pub fn read_grafts(app: &model::ApplicationContext) -> Result<(), errors::GardenError> {
    let root_id = app.get_root_id();
    read_grafts_recursive(app, root_id)
}

/// Read grafts into the specified configuration
pub fn read_grafts_recursive(
    app: &model::ApplicationContext,
    id: model::ConfigId,
) -> Result<(), errors::GardenError> {
    // Defer the recursive calls to avoid an immutable borrow from preventing us from
    // recursively taking an immutable borrow.
    //
    // We build a vector of paths inside an immutable scope and defer construction of
    // the graft Configuration since it requires a mutable borrow against app.
    let mut details = Vec::new();

    // Immutable scope for traversing the configuration.
    {
        let config = app.get_config(id); // Immutable borrow.
        for (graft_name, graft) in &config.grafts {
            let path_str = config.eval_config_path(app, &graft.config);
            let path = std::path::PathBuf::from(&path_str);
            if !path.exists() {
                let config_path = config.get_path()?;
                return Err(errors::GardenError::ConfigurationError(format!(
                    "{}: invalid graft in {:?}",
                    graft.get_name(),
                    config_path
                )));
            }
            let root = if graft.root.is_empty() {
                None
            } else {
                Some(std::path::PathBuf::from(graft.root.clone()))
            };

            details.push((graft_name.clone(), path, root));
        }
    }

    // Read child grafts recursively after the immutable scope has ended.
    for (graft_name, path, root) in details {
        app.add_graft_config(id, &graft_name, &path, root.as_ref())?;
    }

    Ok(())
}
