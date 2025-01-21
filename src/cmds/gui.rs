use anyhow::Result;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::cmds::cmd::CustomOptions;
use crate::{cmd, errors, model, path, query, syntax};

#[derive(Clone)]
enum ModalWindow {
    None,
    Command(String, Vec<model::Variable>),
}

struct GardenApp<'a> {
    app_context: &'a model::ApplicationContext,
    initialized: bool,
    modal_window: ModalWindow,
    modal_window_open: bool,
    query: String,
}

impl GardenApp<'_> {
    /// Display details about a command when right-clicked.
    fn command_details(
        &mut self,
        egui_ctx: &egui::Context,
        command_name: &str,
        command_vec: &Vec<model::Variable>,
    ) {
        let size = egui_ctx.input(|i: &egui::InputState| i.screen_rect());
        let mut value = String::new();
        for cmd in command_vec {
            value.push_str(cmd.get_expr());
            value.push('\n');
        }
        // Open a modal window with the contents of the command.
        let mut text = value.as_str();
        egui::Window::new(command_name)
            .open(&mut self.modal_window_open)
            .default_width(size.width())
            .collapsible(false)
            .resizable(true)
            .movable(true)
            .show(egui_ctx, |ui| {
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
}

impl eframe::App for GardenApp<'_> {
    /// Display and update the user interface.
    fn update(&mut self, egui_ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let num_columns = 4;
        let spacing = 4.0;
        let row_height = 18.0;
        let focus_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::L);

        egui_ctx.set_pixels_per_point(2.0);
        egui_ctx.style_mut(|style| style.visuals.window_shadow = egui::Shadow::NONE);

        if self.modal_window_open {
            match self.modal_window.clone() {
                ModalWindow::None => {
                    self.modal_window_open = false;
                }
                ModalWindow::Command(command_name, command_vec) => {
                    self.command_details(egui_ctx, &command_name, &command_vec);
                }
            }
        }

        egui::CentralPanel::default().show(egui_ctx, |ui| {
            if self.modal_window_open {
                ui.disable();
            }
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Query");
                    ui.add_space(spacing);
                    let query_response = ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Tree query for the gardens, groups or trees to execute commands within")
                    );
                    if !self.initialized {
                        self.initialized = true;
                        ui.memory_mut(|memory| {
                            memory.request_focus(query_response.id);
                        });
                    }
                    if egui_ctx.input_mut(|input| { input.consume_shortcut(&focus_shortcut) }) {
                        ui.memory_mut(|memory| {
                            memory.request_focus(query_response.id);
                        });
                    }
                });

                // Global commands
                ui.separator();
                ui.with_layout(
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.label("Commands");
                    },
                );
                let available_width = ui.available_width();
                let column_width = available_width / (num_columns as f32);
                egui::Grid::new("command_grid")
                    .num_columns(num_columns)
                    .min_col_width(available_width / (num_columns as f32))
                    .show(ui, |ui| {
                    let mut seen_commands = model::StringSet::new();
                    let mut current_column = 0;
                    for (command_name, command_vec) in &self.app_context.get_root_config().commands {
                        let mut command_name = String::from(command_name);
                        if syntax::is_pre_or_post_command(&command_name) {
                            syntax::trim_op_inplace(&mut command_name);
                        }
                        if !seen_commands.insert(command_name.clone()) {
                            continue;
                        }
                        let button_ui = egui::Button::new(&command_name).wrap_mode(egui::TextWrapMode::Wrap);
                        let button = ui.add_sized(egui::Vec2::new(column_width, ui.available_height()), button_ui);
                        if button.clicked() {
                            println!("Running: {}", command_name);
                        }
                        if button.secondary_clicked() {
                            self.modal_window = ModalWindow::Command(command_name.clone(), command_vec.clone());
                            self.modal_window_open = true;
                        }

                        current_column += 1;
                        if current_column % num_columns == 0 {
                            current_column = 0;
                            ui.end_row();
                        }
                    }
                });

                // Query results
                ui.separator();
                let config = self.app_context.get_root_config_mut();
                let query = if self.query.is_empty() {
                    "."
                } else {
                    self.query.as_str()
                };
                let contexts = query::resolve_trees(self.app_context, config, None, query);
                if !contexts.is_empty() {
                    ui.with_layout(
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            ui.label("Tree Query Results");
                        },
                    );
                    ui.separator();
                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
                        .column(Column::auto().at_least(100.0))
                        .column(Column::remainder()
                            .at_least(40.0)
                            .clip(true)
                            .resizable(true))
                        .body(|mut body| {
                            for tree_ctx in &contexts {
                                body.row(row_height, |mut row| {
                                    row.col(|ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.add_space(spacing);
                                                ui.monospace(&tree_ctx.tree);
                                            },
                                        );
                                    });
                                    let tree = config.get_tree(&tree_ctx.tree);
                                    if let Some(Ok(path)) = tree.map(|tree| tree.path_as_ref()) {
                                        row.col(|ui| {
                                            ui.monospace(path);
                                        });
                                    }
                                });
                            }
                        }
                    );
                }
            });
        });
    }
}

/// Main entry point for `garden gui <query> <command>...`.
pub fn main(app_context: &model::ApplicationContext, options: &CustomOptions) -> Result<()> {
    let egui_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([892.0, 480.0]),
        ..Default::default()
    };
    let current_directory = path::current_dir();
    let basename = current_directory
        .file_name()
        .and_then(|os_name| os_name.to_str())
        .unwrap_or(".");
    let window_title = format!("Garden - {}", basename);
    let app_state = GardenApp {
        app_context,
        initialized: false,
        modal_window_open: false,
        modal_window: ModalWindow::None,
        query: options.query_string(),
    };
    let result = eframe::run_native(
        &window_title,
        egui_options,
        Box::new(|_| Ok(Box::new(app_state))),
    );
    if result.is_err() {
        cmd::result_from_exit_status(errors::EX_ERROR).map_err(|err| err.into())
    } else {
        Ok(())
    }
}
