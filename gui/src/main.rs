use anyhow::Result;
use clap::{Parser, ValueHint};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use garden::cli::GardenOptions;
use garden::{cli, cmd, constants, display, errors, model, path, query, string, syntax};

const NUM_COMMAND_COLUMNS: usize = 4;

/// Return the max of two floats.
macro_rules! max {
    // We could use the float_ord crate to get std:::cmp::max(f32, f32) working,
    // but this is simpler.
    ($a: expr) => ($a);
    ($a: expr, $($b: expr),+) => {{
        let b = max!($($b),*);
        if $a > b {
            $a
        } else {
            b
        }
    }}
}

/// Main entry point for the "garden-gui" command.
fn main() -> Result<()> {
    let mut gui_options = GuiOptions::parse();
    // The color mode is modified by update() but we don't need to care about its
    // new value because update() ends up modifying global state that is ok to leave
    // alone after the call to update(). We restore the value of color so that we can
    // pass the original command-line value along to spawned garden commands.
    let color = gui_options.color.clone();
    gui_options.update();
    gui_options.color = color;

    cmd::initialize_threads_option(gui_options.num_jobs)?;

    let options = gui_options.to_main_options();
    let app = model::ApplicationContext::from_options(&options)?;
    // Return the appropriate exit code when a GardenError is encountered.
    if let Err(err) = gui_main(&app, &gui_options) {
        let exit_status = errors::exit_status_from_error(err);
        std::process::exit(exit_status);
    }

    Ok(())
}

/// Main entry point for `garden gui`.
fn gui_main(app_context: &model::ApplicationContext, options: &GuiOptions) -> Result<()> {
    let egui_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([892.0, 480.0]),
        ..Default::default()
    };
    let current_directory = path::current_dir();
    let basename = current_directory
        .file_name()
        .and_then(|os_name| os_name.to_str())
        .unwrap_or(".");
    let window_title = format!("Garden - {basename}");
    let query = options.query_string();
    let (send_command, recv_command) = crossbeam::channel::unbounded();

    let app_state = GardenApp {
        app_context,
        initialized: false,
        modal_window_open: false,
        modal_window: ModalWindow::None,
        options: options.clone(),
        query,
        send_command: send_command.clone(),
        view_metrics: ViewMetrics {
            spacing: 4.0,
            row_height: 18.0,
            window_margin: 24.0,
        },
    };

    let command_thread = std::thread::spawn(move || loop {
        match recv_command.recv() {
            Ok(CommandMessage::GardenCommand(command)) => {
                display::print_command_string_vec(&command);
                let exec = cmd::exec_cmd(&command);
                let result = cmd::subprocess_result(exec.join());
                if result == Err(errors::EX_UNAVAILABLE) {
                    eprintln!("error: garden is not installed");
                    eprintln!("error: run \"cargo install garden-tools\"");
                }
            }
            Ok(CommandMessage::Quit) => break,
            Err(_) => break,
        }
    });

    let result = eframe::run_native(
        &window_title,
        egui_options,
        Box::new(|_| Ok(Box::new(app_state))),
    );

    // Tell the command thread to quit.
    send_command.send(CommandMessage::Quit).unwrap_or(());
    command_thread.join().unwrap_or(());

    result.map_err(|_| errors::error_from_exit_status(errors::EX_ERROR).into())
}

/// Run the Garden graphical interface
#[derive(Parser, Clone, Debug)]
#[command(bin_name = constants::GARDEN_GUI)]
#[command(author, version, about, long_about = None)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
pub(crate) struct GuiOptions {
    /// Change directories before searching for Garden files
    #[arg(long, short = 'C', value_hint = ValueHint::DirPath)]
    pub chdir: Option<std::path::PathBuf>,
    /// Use ANSI colors [auto, true, false, on, off, always, never, 1, 0]
    #[arg(
        long,
        require_equals = true,
        num_args = 0..=1,
        default_value_t = model::ColorMode::Auto,
        default_missing_value = "true",
        value_name = "WHEN",
        value_parser = model::ColorMode::parse_from_str,
    )]
    pub color: model::ColorMode,
    /// Set the Garden file to use
    #[arg(long, short, value_hint = ValueHint::FilePath)]
    pub config: Option<std::path::PathBuf>,
    /// Increase verbosity for a debug category
    #[arg(long, short, action = clap::ArgAction::Append)]
    pub debug: Vec<String>,
    /// Set variables using 'name=value' expressions
    #[arg(long, short = 'D')]
    define: Vec<String>,
    /// Perform a trial run without running commands
    #[arg(long, short = 'N')]
    dry_run: bool,
    /// Continue to the next tree when errors occur
    #[arg(long, short)]
    keep_going: bool,
    /// Do not pass "-e" to the shell.
    /// Prevent the "errexit" shell option from being set. By default, the "-e" option
    /// is passed to the configured shell so that multi-line and multi-statement
    /// commands halt execution when the first statement with a non-zero exit code is
    /// encountered. This option has the effect of making multi-line and
    /// multi-statement commands run all statements even when an earlier statement
    /// returns a non-zero exit code.
    #[arg(long = "no-errexit", short = 'n', default_value_t = true, action = clap::ArgAction::SetFalse)]
    exit_on_error: bool,
    /// Run commands even when the tree does not exist.
    #[arg(long, short)]
    force: bool,
    /// Run commands in parallel using the specified number of jobs.
    #[arg(
        long = "jobs",
        short = 'j',
        require_equals = false,
        num_args = 0..=1,
        default_missing_value = "0",
        value_name = "JOBS",
    )]
    num_jobs: Option<usize>,
    /// Be quiet
    #[arg(short, long)]
    quiet: bool,
    /// Increase verbosity level (default: 0)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// Do not pass "-o shwordsplit" to zsh.
    /// Prevent the "shwordsplit" shell option from being set when using zsh.
    /// The "-o shwordsplit" option is passed to zsh by default so that unquoted
    /// $variable expressions are subject to word splitting, just like other shells.
    /// This option disables this behavior.
    #[arg(long = "no-wordsplit", short = 'z', default_value_t = true, action = clap::ArgAction::SetFalse)]
    word_split: bool,
    /// Set the Garden tree root
    #[arg(long, short, value_hint = ValueHint::DirPath)]
    pub root: Option<std::path::PathBuf>,
    /// Tree queries for the Gardens/Groups/Trees to execute commands within
    #[arg(last = true)]
    queries: Vec<String>,
}

impl GardenOptions for GuiOptions {
    fn get_chdir(&self) -> &Option<std::path::PathBuf> {
        &self.chdir
    }

    fn get_color_mut(&mut self) -> &mut model::ColorMode {
        &mut self.color
    }

    fn get_config(&self) -> &Option<std::path::PathBuf> {
        &self.config
    }

    fn set_config(&mut self, config: std::path::PathBuf) {
        self.config = Some(config);
    }

    fn get_debug(&self) -> &[String] {
        &self.debug
    }

    fn get_root(&self) -> &Option<std::path::PathBuf> {
        &self.root
    }

    fn set_root(&mut self, root: std::path::PathBuf) {
        self.root = Some(root);
    }
}

impl GuiOptions {
    /// Return the queries as a single string.
    fn query_string(&self) -> String {
        shell_words::join(&self.queries)
    }

    /// Convert GuiOptions to MainOptions
    fn to_main_options(&self) -> cli::MainOptions {
        let arguments = cli::Arguments::default();
        cli::MainOptions {
            chdir: self.chdir.clone(),
            command: cli::Command::Gui(arguments),
            color: self.color.clone(),
            config: self.config.clone(),
            debug: self.debug.clone(),
            define: self.define.clone(),
            quiet: self.quiet,
            verbose: self.verbose,
            root: self.root.clone(),
        }
    }
}

#[derive(Clone)]
enum ModalWindow {
    None,
    Command(String, Vec<model::Variable>),
    Grow(Vec<String>),
    List(Vec<String>),
}

enum CommandMessage {
    GardenCommand(Vec<String>),
    Quit,
}

struct ViewMetrics {
    spacing: f32,
    row_height: f32,
    window_margin: f32,
}

struct GardenApp<'a> {
    app_context: &'a model::ApplicationContext,
    initialized: bool,
    modal_window: ModalWindow,
    modal_window_open: bool,
    options: GuiOptions,
    query: String,
    send_command: crossbeam::channel::Sender<CommandMessage>,
    view_metrics: ViewMetrics,
}

/// Return the base command arguments for any garden sub-command.
fn get_garden_command_vec(
    options: &GuiOptions,
    command_name: &str,
    query: &str,
) -> (Vec<String>, Vec<String>) {
    let queries = cmd::shlex_split(query);
    let capacity = get_command_capacity(options, &queries);
    let mut command = Vec::with_capacity(capacity);
    command.push(constants::GARDEN.to_string());

    if options.color != model::ColorMode::Auto {
        command.push(format!("--color={}", options.color));
    }
    if let Some(config) = &options.config {
        command.push(format!("--config={}", config.to_string_lossy()));
    }
    for debug in &options.debug {
        command.push(format!("--debug={debug}"));
    }
    if let Some(root) = &options.root {
        command.push(format!("--root={}", root.to_string_lossy()));
    }
    if options.verbose > 0 {
        let verbose = cli::verbose_string(options.verbose);
        command.push(verbose);
    }
    // Custom command name.
    command.push(command_name.to_string());

    (command, queries)
}

/// Calculate a "garden" command for running the specified command.
fn get_custom_command_vec(options: &GuiOptions, command_name: &str, query: &str) -> Vec<String> {
    let (mut command, mut queries) = get_garden_command_vec(options, command_name, query);
    // Options after this point are supported by "garden <command> [options]".
    for define in &options.define {
        command.push(string!("--define"));
        command.push(define.to_string());
    }
    if options.dry_run {
        command.push(string!("--dry-run"));
    }
    if options.force {
        command.push(string!("--force"));
    }
    if options.keep_going {
        command.push(string!("--keep-going"));
    }
    if let Some(num_jobs) = &options.num_jobs {
        command.push(format!("--jobs={num_jobs}"));
    }
    if !options.exit_on_error {
        command.push(string!("--no-errexit"));
    }
    if !options.word_split {
        command.push(string!("--no-wordsplit"));
    }
    if options.quiet {
        command.push(string!("--quiet"));
    }

    // Query positional arguments
    command.append(&mut queries);

    command
}

/// Calculate a "garden grow" command.
fn get_grow_command_vec(options: &GuiOptions, query: &str) -> Vec<String> {
    let (mut command, mut queries) = get_garden_command_vec(options, "grow", query);
    // Query positional arguments
    if query.is_empty() {
        command.push(string!("."));
    } else {
        command.append(&mut queries);
    }

    command
}

/// Calculate a "garden ls" command.
fn get_ls_command_vec(options: &GuiOptions, query: &str) -> Vec<String> {
    let (mut command, mut queries) = get_garden_command_vec(options, "ls", query);
    // garden ls --no-commands --no-groups --no-gardens <query>
    command.push(string!("-CGN"));
    // Query positional arguments
    if !query.is_empty() {
        command.append(&mut queries);
    }

    command
}

/// Calculate the vector capacity for custom command storage.
fn get_command_capacity(options: &GuiOptions, queries: &[String]) -> usize {
    let mut size = 2; // garden <cmd>
    size += queries.len();
    size += options.define.len();
    size += options.debug.len() * 2;
    if options.dry_run {
        size += 1;
    }
    if options.config.is_some() {
        size += 1;
    }
    if options.color != model::ColorMode::Auto {
        size += 1;
    }
    if !options.exit_on_error {
        size += 1;
    }
    if options.force {
        size += 1;
    }
    if options.keep_going {
        size += 1;
    }
    if options.quiet {
        size += 1;
    }
    if options.root.is_some() {
        size += 1;
    }
    if options.verbose > 0 {
        size += 1;
    }
    if !options.word_split {
        size += 1;
    }
    if options.num_jobs.is_some() {
        size += 1;
    }

    size
}

impl GardenApp<'_> {
    /// Add the query bar.
    fn display_query_input(&mut self, egui_ctx: &egui::Context, ui: &mut egui::Ui) {
        let focus_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::L);
        ui.horizontal(|ui| {
            ui.label("Query");
            ui.add_space(self.view_metrics.spacing);
            let completions = completion_values(self.app_context);
            let text_edit =
                egui_autocomplete::AutoCompleteTextEdit::new(&mut self.query, &completions)
                    .multiple_words(true)
                    .set_text_edit_properties(|text_edit: egui::TextEdit<'_>| {
                        text_edit.hint_text(
                        "Tree query for the gardens, groups or trees to execute commands within",
                    )
                    });
            let query_response = ui.add_sized(ui.available_size(), text_edit);
            if !self.initialized {
                self.initialized = true;
                ui.memory_mut(|memory| {
                    memory.request_focus(query_response.id);
                });
            }
            if egui_ctx.input_mut(|input| input.consume_shortcut(&focus_shortcut)) {
                ui.memory_mut(|memory| {
                    memory.request_focus(query_response.id);
                });
            }
        });
    }

    /// Add the command grid.
    fn display_commands(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.label("Commands");
        });
        let available_width = max!(
            100.0,
            ui.available_width() - self.view_metrics.window_margin
        );
        let column_width = available_width / (NUM_COMMAND_COLUMNS as f32);
        egui::Grid::new("command_grid")
            .num_columns(NUM_COMMAND_COLUMNS)
            .min_col_width(column_width)
            .max_col_width(column_width)
            .show(ui, |ui| {
                let mut seen_commands = model::StringSet::new();
                // Add a "grow" command that runs against the entire tree query.
                let button_ui = egui::Button::new("grow").wrap_mode(egui::TextWrapMode::Wrap);
                let button = ui.add_sized(
                    egui::Vec2::new(column_width, ui.available_height()),
                    button_ui,
                );
                if button.clicked() {
                    let command_vec = get_grow_command_vec(&self.options, &self.query);
                    self.send_command
                        .send(CommandMessage::GardenCommand(command_vec))
                        .unwrap_or(());
                }
                if button.secondary_clicked() {
                    let command_vec = get_grow_command_vec(&self.options, &self.query);
                    self.modal_window = ModalWindow::Grow(command_vec.clone());
                    self.modal_window_open = true;
                }

                // Add an "ls" command that runs against the entire tree query.
                let button_ui = egui::Button::new("ls").wrap_mode(egui::TextWrapMode::Wrap);
                let button = ui.add_sized(
                    egui::Vec2::new(column_width, ui.available_height()),
                    button_ui,
                );
                if button.clicked() {
                    let command_vec = get_ls_command_vec(&self.options, &self.query);
                    self.send_command
                        .send(CommandMessage::GardenCommand(command_vec))
                        .unwrap_or(());
                }
                if button.secondary_clicked() {
                    let command_vec = get_ls_command_vec(&self.options, &self.query);
                    self.modal_window = ModalWindow::Grow(command_vec.clone());
                    self.modal_window_open = true;
                }

                let mut current_column = 2;
                for (command_name, command_vec) in &self.app_context.get_root_config().commands {
                    self.add_command_button(
                        ui,
                        command_name,
                        command_vec,
                        &mut seen_commands,
                        column_width,
                        &mut current_column,
                    );
                }

                // Resolve the tree query so that we can add commands specific to the query.
                let contexts = query_trees(self.app_context, &self.query);
                // Insert commands from tree contexts with garden scopes.
                let mut seen_gardens = model::StringSet::new();
                for context in &contexts {
                    let Some(garden_name) = context.garden.as_ref() else {
                        continue;
                    };
                    // If we've already seen this garden then we can skip adding its commands.
                    if !seen_gardens.insert(garden_name.clone()) {
                        return;
                    }
                    let Some(garden) = self.app_context.get_root_config().get_garden(garden_name)
                    else {
                        continue;
                    };
                    for (command_name, command_vec) in &garden.commands {
                        self.add_command_button(
                            ui,
                            command_name,
                            command_vec,
                            &mut seen_commands,
                            column_width,
                            &mut current_column,
                        );
                    }
                }

                // Insert tree-specific commands from each tree context.
                for context in &contexts {
                    let Some(tree) = self.app_context.get_root_config().get_tree(&context.tree)
                    else {
                        continue;
                    };
                    for (command_name, command_vec) in &tree.commands {
                        self.add_command_button(
                            ui,
                            command_name,
                            command_vec,
                            &mut seen_commands,
                            column_width,
                            &mut current_column,
                        );
                    }
                }
            });
    }

    /// Add a command button to the command grid.
    #[inline]
    fn add_command_button(
        &mut self,
        ui: &mut egui::Ui,
        command_name: &str,
        command_vec: &[model::Variable],
        seen_commands: &mut model::StringSet,
        column_width: f32,
        current_column: &mut usize,
    ) {
        let mut command_name = String::from(command_name);
        if syntax::is_pre_or_post_command(&command_name) {
            syntax::trim_op_inplace(&mut command_name);
        }
        if !seen_commands.insert(command_name.clone()) {
            return;
        }
        let button_ui = egui::Button::new(&command_name).wrap_mode(egui::TextWrapMode::Wrap);
        let button = ui.add_sized(
            egui::Vec2::new(column_width, ui.available_height()),
            button_ui,
        );
        if button.clicked() {
            let command_vec = get_custom_command_vec(&self.options, &command_name, &self.query);
            self.send_command
                .send(CommandMessage::GardenCommand(command_vec.to_vec()))
                .unwrap_or(());
        }
        if button.secondary_clicked() {
            self.modal_window = ModalWindow::Command(command_name.clone(), command_vec.to_vec());
            self.modal_window_open = true;
        }

        *current_column += 1;
        if (*current_column).is_multiple_of(NUM_COMMAND_COLUMNS) {
            *current_column = 0;
            ui.end_row();
        }
    }

    /// Display details about a custom command when right-clicked.
    fn custom_command_details(
        &mut self,
        egui_ctx: &egui::Context,
        command_name: &str,
        command_vec: &Vec<model::Variable>,
    ) {
        let mut value = String::new();
        for cmd in command_vec {
            value.push_str(cmd.get_expr());
            value.push('\n');
        }
        self.command_string_window(egui_ctx, command_name, &value);
    }

    /// Display details about a "garden grow" command when right-clicked.
    fn grow_command_details(&mut self, egui_ctx: &egui::Context, command_vec: &[String]) {
        let value = shell_words::join(command_vec);
        self.command_string_window(egui_ctx, "grow", &value);
    }

    /// Display details about a "garden ls" command when right-clicked.
    fn ls_command_details(&mut self, egui_ctx: &egui::Context, command_vec: &[String]) {
        let value = shell_words::join(command_vec);
        self.command_string_window(egui_ctx, "ls", &value);
    }

    /// Display details about a command when right-clicked
    fn command_string_window(&mut self, egui_ctx: &egui::Context, command_name: &str, value: &str) {
        let size = egui_ctx.input(|i: &egui::InputState| i.content_rect());
        // Open a modal window with the contents of the command.
        let mut text = value;
        let modal_window_open = self.modal_window_open;
        let close_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::W);
        let esc_shortcut =
            egui::KeyboardShortcut::new(egui::Modifiers::default(), egui::Key::Escape);
        let esc_shortcut_alt =
            egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::OpenBracket);
        egui::Window::new(command_name)
            .open(&mut self.modal_window_open)
            .default_width(size.width())
            .collapsible(false)
            .resizable(true)
            .movable(true)
            .show(egui_ctx, |ui| {
                if modal_window_open {
                    if egui_ctx.input_mut(|input| input.consume_shortcut(&close_shortcut)) {
                        self.modal_window = ModalWindow::None;
                    }
                    if egui_ctx.input_mut(|input| input.consume_shortcut(&esc_shortcut)) {
                        self.modal_window = ModalWindow::None;
                    }
                    if egui_ctx.input_mut(|input| input.consume_shortcut(&esc_shortcut_alt)) {
                        self.modal_window = ModalWindow::None;
                    }
                }
                ui.vertical(|ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut text)
                            .code_editor()
                            .desired_width(f32::INFINITY),
                    );
                    ui.with_layout(
                        egui::Layout::default().with_cross_align(egui::Align::RIGHT),
                        |ui| {
                            if ui.button("Close").clicked() {
                                self.modal_window = ModalWindow::None;
                            }
                        },
                    );
                });
            });
    }

    /// Add the variables table.
    fn display_variables(&mut self, ui: &mut egui::Ui) {
        if self.app_context.get_root_config().variables.is_empty() {
            return;
        }
        ui.separator();
        ui.collapsing("Variables", |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
                .column(Column::auto().at_least(100.0))
                .column(
                    Column::remainder()
                        .at_least(40.0)
                        .clip(true)
                        .resizable(true),
                )
                .body(|mut body| {
                    for (name, variable) in &self.app_context.get_root_config().variables {
                        body.row(self.view_metrics.row_height, |mut row| {
                            row.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(self.view_metrics.spacing);
                                        ui.monospace(name);
                                    },
                                );
                            });
                            row.col(|ui| {
                                ui.monospace(variable.get_expr());
                            });
                        });
                    }
                });
        });
    }

    //// Add the `--defines` overrides table.
    fn display_override_variables(&mut self, ui: &mut egui::Ui) {
        if self
            .app_context
            .get_root_config()
            .override_variables
            .is_empty()
        {
            return;
        }
        ui.separator();
        ui.collapsing("Defines", |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
                .column(Column::auto().at_least(100.0))
                .column(
                    Column::remainder()
                        .at_least(40.0)
                        .clip(true)
                        .resizable(true),
                )
                .body(|mut body| {
                    for (name, variable) in &self.app_context.get_root_config().override_variables {
                        body.row(self.view_metrics.row_height, |mut row| {
                            row.col(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_space(self.view_metrics.spacing);
                                        ui.monospace(name);
                                    },
                                );
                            });
                            row.col(|ui| {
                                ui.monospace(variable.get_expr());
                            });
                        });
                    }
                });
        });
    }

    /// Add the query results table
    fn display_query_results(&mut self, ui: &mut egui::Ui) {
        let config = self.app_context.get_root_config();
        let contexts = query_trees(self.app_context, &self.query);
        if contexts.is_empty() {
            return;
        }

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.label("Tree Query Results");
        });
        ui.separator();

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
            .resizable(true)
            .column(Column::auto().clip(true))
            .column(Column::remainder().clip(true))
            .body(|mut body| {
                let mut seen_trees  = model::StringSet::new();
                for tree_ctx in &contexts {
                    // If we've already seen this tree then we can skip it.
                    if !seen_trees.insert(tree_ctx.tree.clone()) {
                        continue;
                    }
                    let Some(tree) = config.get_tree(&tree_ctx.tree) else {
                        continue;
                    };
                    let Ok(path) = tree.path_as_ref() else {
                        continue;
                    };
                    body.row(self.view_metrics.row_height, |mut row| {
                        row.col(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let copy_button = ui.button("📋").on_hover_ui(|ui| {
                                        ui.label("Click to copy path. Right-click to copy the tree name.");
                                    });
                                    if copy_button.clicked() {
                                        let copy_text = egui::OutputCommand::CopyText(path.to_string());
                                        ui.output_mut(|output| output.commands.push(copy_text));
                                    }
                                    if copy_button.secondary_clicked() {
                                        let copy_text = egui::OutputCommand::CopyText(tree_ctx.tree.to_string());
                                        ui.output_mut(|output| output.commands.push(copy_text));
                                    }
                                    ui.monospace(&tree_ctx.tree);
                                },
                            );
                        });
                        row.col(|ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    // Fields are added in reverse order to get "<path> [grow] [ls]".
                                    let ls_button_ui = egui::Button::new("ls");
                                    let ls_button = ui.add(ls_button_ui);
                                    if ls_button.clicked() {
                                        let tree_query = format!("@{}", tree_ctx.tree);
                                        let command_vec =
                                            get_ls_command_vec(&self.options, &tree_query);
                                        self.send_command
                                            .send(CommandMessage::GardenCommand(command_vec))
                                            .unwrap_or(());
                                    }
                                    if ls_button.secondary_clicked() {
                                        let tree_query = format!("@{}", tree_ctx.tree);
                                        let command_vec =
                                            get_ls_command_vec(&self.options, &tree_query);
                                        self.modal_window = ModalWindow::List(command_vec);
                                        self.modal_window_open = true;
                                    }
                                    // Add the "grow" button.
                                    ui.add_space(self.view_metrics.spacing);
                                    let grow_button_ui = egui::Button::new("grow");
                                    let grow_button = ui.add(grow_button_ui);
                                    if grow_button.clicked() {
                                        let tree_query = format!("@{}", tree_ctx.tree);
                                        let command_vec =
                                            get_grow_command_vec(&self.options, &tree_query);
                                        self.send_command
                                            .send(CommandMessage::GardenCommand(command_vec))
                                            .unwrap_or(());
                                    }
                                    if grow_button.secondary_clicked() {
                                        let tree_query = format!("@{}", tree_ctx.tree);
                                        let command_vec =
                                            get_grow_command_vec(&self.options, &tree_query);
                                        self.modal_window = ModalWindow::Grow(command_vec);
                                        self.modal_window_open = true;
                                    }
                                    ui.add_space(self.view_metrics.spacing);
                                    if std::path::PathBuf::from(path).exists() {
                                        if ui.monospace(path).on_hover_ui(|ui| {
                                            ui.label("Right-click to copy path.");
                                        }).secondary_clicked() {
                                            let copy_text = eframe::egui::OutputCommand::CopyText(path.to_string());
                                            ui.output_mut(|output| output.commands.push(copy_text));
                                        }
                                    } else if ui.label(
                                        egui::RichText::new(path)
                                            .monospace()
                                            .color(egui::Color32::RED),
                                    ).on_hover_ui(|ui| {
                                        ui.label("Right-click to copy path.");
                                    }).secondary_clicked() {
                                        let copy_text = eframe::egui::OutputCommand::CopyText(path.to_string());
                                        ui.output_mut(|output| output.commands.push(copy_text));
                                    }
                                },
                            );
                        });
                    });
                }
            }
        );
    }
}

impl eframe::App for GardenApp<'_> {
    /// Display the Garden GUI window.
    fn update(&mut self, egui_ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_ctx.set_pixels_per_point(2.0);
        egui_ctx.style_mut(|style| style.visuals.window_shadow = egui::Shadow::NONE);

        if self.modal_window_open {
            match self.modal_window.clone() {
                ModalWindow::None => {
                    self.modal_window_open = false;
                }
                ModalWindow::Command(command_name, command_vec) => {
                    self.custom_command_details(egui_ctx, &command_name, &command_vec);
                }
                ModalWindow::Grow(command_vec) => {
                    self.grow_command_details(egui_ctx, &command_vec);
                }
                ModalWindow::List(command_vec) => {
                    self.ls_command_details(egui_ctx, &command_vec);
                }
            }
        }

        let quit_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Q);
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            // Close the window when the quit_shortcut is triggered.
            if egui_ctx.input_mut(|input| input.consume_shortcut(&quit_shortcut)) {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
            if self.modal_window_open {
                ui.disable();
            }
            ui.vertical(|ui| {
                self.display_query_input(egui_ctx, ui);
                ui.separator();
                self.display_commands(ui);
                self.display_variables(ui);
                self.display_override_variables(ui);
                ui.separator();
                self.display_query_results(ui);
            });
        });
    }
}

/// Resolve multiple tree queries contained within a single string that uses
/// shell syntax for specifying multiple tokens.
#[inline]
fn query_trees(app_context: &model::ApplicationContext, query: &str) -> Vec<model::TreeContext> {
    let query_str = if query.is_empty() { "." } else { query };
    let config = app_context.get_root_config();
    let mut contexts = Vec::with_capacity(config.trees.len());
    let queries = cmd::shlex_split(query_str);
    for query in &queries {
        contexts.append(&mut query::resolve_trees(app_context, config, None, query));
    }

    contexts
}

/// Provide candidate completion values for the tree query input field.
#[inline]
fn completion_values(app_context: &model::ApplicationContext) -> Vec<String> {
    let config = app_context.get_root_config();
    let mut results = Vec::with_capacity(
        config.trees.len() * 2 + config.gardens.len() * 2 + config.groups.len() * 2,
    );
    let mut tree_results = Vec::with_capacity(config.trees.len());
    let mut group_results = Vec::with_capacity(config.groups.len());
    let mut garden_results = Vec::with_capacity(config.gardens.len());

    for (name, _tree) in &config.trees {
        results.push(name.clone());
        tree_results.push(format!("@{name}"));
    }
    for (name, _group) in &config.groups {
        results.push(name.clone());
        group_results.push(format!("%{name}"));
    }
    for (name, _garden) in &config.gardens {
        results.push(name.clone());
        garden_results.push(format!(":{name}"));
    }

    results.sort();
    tree_results.sort();
    group_results.sort();
    garden_results.sort();
    results.append(&mut tree_results);
    results.append(&mut group_results);
    results.append(&mut garden_results);

    results
}
