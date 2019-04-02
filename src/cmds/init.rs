use ::cmd;
use ::config;
use ::model;

struct InitOptions {
    pub dirname: std::path::PathBuf,
    pub filename: String,
    pub force: bool,
    pub global: bool,
    pub root: String,
}

impl std::default::Default for InitOptions {
    fn default() -> Self {
        InitOptions {
            dirname: std::env::current_dir().unwrap(),
            filename: "garden.yaml".to_string(),
            force: false,
            global: false,
            root: "${GARDEN_CONFIG_DIR}".to_string(),
        }
    }
}


pub fn main(options: &mut model::CommandOptions) -> i32 {

    let mut init_options = InitOptions::default();
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("garden init - initialize a new garden.yaml");

        ap.refer(&mut init_options.global)
            .add_option(&["--global"], argparse::StoreTrue,
                        "use the user-wide configuration directory
                        (~/.config/garden/garden.yaml)");

        ap.refer(&mut init_options.force)
            .add_option(&["-f", "--force"], argparse::StoreTrue,
                        "overwrite existing config files");

        ap.refer(&mut init_options.root)
            .add_option(&["-r", "--root"], argparse::Store,
                        "specify the garden root (default: ${GARDEN_CONFIG_DIR}");

        ap.refer(&mut init_options.filename)
            .add_argument("filename", argparse::Store,
                          "config file to write (default: garden.yaml)");

        options.args.insert(0, "garden init".to_string());
        return_on_err!(ap.parse(options.args.to_vec(),
                                &mut std::io::stdout(),
                                &mut std::io::stderr()));
    }

    init(options, &mut init_options)
}


fn init(
    options: &model::CommandOptions,
    init_options: &mut InitOptions,
) -> i32 {

    let file_path = std::path::PathBuf::from(&init_options.filename);
    if file_path.is_absolute() {
        if init_options.global {
            errmsg!("'--global' cannot be used with an absolute path");
            return cmd::ExitCode::Usage.into();
        }

        init_options.dirname = file_path
            .parent().as_ref().unwrap().to_path_buf();
        init_options.filename = file_path
            .file_name().as_ref().unwrap().to_string_lossy().into();
    }
    if init_options.global {
        init_options.dirname = config::xdg_dir();
    }

    let mut config_path = init_options.dirname.to_path_buf();
    config_path.push(&init_options.filename);

    if !init_options.force && config_path.exists() {
        errmsg!("{:?} already exists, use \"--force\" to overwrite",
                config_path.to_string_lossy());
        return cmd::ExitCode::FileExists.into();
    }

    cmd::ExitCode::Success.into()
}
