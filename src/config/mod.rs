use super::errors;
use super::model;
use super::model::ConfigId;
use super::path;

/// YAML reader
pub mod reader;

/// YAML writer
pub mod writer;

// Search for configuration in the following locations:
//  .
//  ./garden
//  ./etc/garden
//  ~/.config/garden
//  ~/etc/garden
//  /etc/garden

fn search_path() -> Vec<std::path::PathBuf> {
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

pub fn new(
    config: &Option<std::path::PathBuf>,
    root: &str,
    config_verbose: u8,
    parent: Option<ConfigId>,
) -> Result<model::Configuration, errors::GardenError> {
    let mut cfg = model::Configuration::new();
    if let Some(parent_id) = parent {
        cfg.set_parent(parent_id);
    }
    cfg.verbose = config_verbose;

    // Override the configured garden root
    if !root.is_empty() {
        cfg.root.set_expr(root.to_string());
    }

    let mut basename: String = "garden.yaml".into();

    // Find garden.yaml in the search path
    let mut found = false;
    if let Some(config_path) = config {
        if config_path.is_file() || config_path.is_absolute() {
            // If an absolute path was specified, or if the file exists,
            // short-circuit the search; the config file might be missing but
            // we shouldn't silently use a different config file.
            cfg.set_path(config_path.to_path_buf());
            found = true;
        } else {
            // The specified path is a basename or relative path to be found
            // in the config search path.
            basename = config_path.to_string_lossy().into();
        }
    }

    if !found {
        for entry in search_path() {
            let mut candidate = entry.to_path_buf();
            candidate.push(basename.clone());
            if candidate.exists() {
                cfg.set_path(candidate);
                found = true;
                break;
            }
        }
    }
    if config_verbose > 0 {
        debug!(
            "config: path: {:?}, root: {:?}, found: {}",
            cfg.path, cfg.root, found
        );
    }

    if found {
        // Read file contents.
        let config_path = cfg.get_path()?;
        if let Ok(config_string) = std::fs::read_to_string(config_path) {
            parse(&config_string, config_verbose, &mut cfg)?;
        } else {
            // Return a default Configuration If we are unable to read the file.
            return Ok(cfg);
        }
    }

    // Default to the current directory when garden.root is unspecified
    if cfg.root.get_expr().is_empty() {
        cfg.root.set_expr(path::current_dir_string());
    }

    Ok(cfg)
}

/// Read configuration from a path.  Wraps new() to make the path required..
pub fn from_path(
    path: std::path::PathBuf,
    root: &str,
    config_verbose: u8,
    parent: Option<ConfigId>,
) -> Result<model::Configuration, errors::GardenError> {
    new(&Some(path), root, config_verbose, parent)
}

/// Read configuration from a path string.  Wraps from_path() to simplify usage.
pub fn from_path_string(
    path: &str,
    verbose: u8,
) -> Result<model::Configuration, errors::GardenError> {
    from_path(std::path::PathBuf::from(path), "", verbose, None)
}

/// Create a model::Configuration instance from model::CommandOptions
pub fn from_options(
    options: &model::CommandOptions,
) -> Result<model::Configuration, errors::GardenError> {
    let config_verbose = options.debug_level("config");
    let mut config = new(&options.filename, &options.root, config_verbose, None)?;

    if config.path.is_none() {
        error!("unable to find a configuration file -- use --config <path>");
    }
    if config_verbose > 1 {
        eprintln!("config: {:?}", config.get_path()?);
    }
    if config_verbose > 2 {
        debug!("{}", config);
    }

    for key in &options.debug {
        let current = *config.debug.get(key).unwrap_or(&0);
        config.debug.insert(key.into(), current + 1);
    }

    for k_eq_v in &options.variables {
        let name: String;
        let expr: String;
        let values: Vec<&str> = k_eq_v.splitn(2, '=').collect();
        if values.len() == 1 {
            name = values[0].into();
            expr = "".into();
        } else if values.len() == 2 {
            name = values[0].into();
            expr = values[1].into();
        } else {
            error!("unable to split '{}'", k_eq_v);
        }
        config
            .variables
            .insert(0, model::NamedVariable::new(name, expr, None));
    }

    Ok(config)
}

/// Parse and apply configuration from a YAML/JSON string
pub fn parse(
    config_string: &str,
    verbose: u8,
    cfg: &mut model::Configuration,
) -> Result<(), errors::GardenError> {
    reader::parse(config_string, verbose, cfg)?;
    // Initialize the configuration now that the values have been read.
    cfg.initialize();

    Ok(())
}

/// Read grafts into the root configuration on down.
pub fn read_grafts(app: &mut model::ApplicationContext) -> Result<(), errors::GardenError> {
    let root_id = app.get_root_id();
    read_grafts_recursive(app, root_id)
}

/// Read grafts into the specified configuration
fn read_grafts_recursive(
    app: &mut model::ApplicationContext,
    id: ConfigId,
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
        for (idx, graft) in config.grafts.iter().enumerate() {
            let path_str = config.eval_config_path(&graft.config);
            let path = std::path::PathBuf::from(&path_str);
            if !path.exists() {
                let config_path = config.get_path()?;
                return Err(errors::GardenError::ConfigurationError(format!(
                    "{}: invalid graft in {:?}",
                    graft.get_name(),
                    config_path
                )));
            }
            details.push((idx, path, graft.root.to_string()));
        }
    }

    // Read child grafts recursively after the immutable scope has ended.
    let config_verbose = app.options.debug_level("config");
    for (idx, path, root) in details {
        // Read the Configuration referenced by the graft.
        let graft_config = from_path(path, &root, config_verbose, Some(id))?;
        // The app Arena takes ownershp of the Configuration.
        let graft_id = app.add_graft(id, graft_config);
        // Record the config ID in the graft structure.
        app.get_config_mut(id).grafts[idx].set_id(graft_id);
        // Read child grafts recursively.
        read_grafts_recursive(app, graft_id)?;
    }

    Ok(())
}
