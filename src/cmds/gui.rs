use anyhow::Result;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

use crate::cmds::cmd::CustomOptions;
use crate::{cmd, errors, model, path, query, syntax};

struct GardenApp<'a> {
    query: String,
    app_context: &'a model::ApplicationContext,
}

impl GardenApp<'_> {
    /// Display details about a command when right-clicked.
    fn command_button_details(&self, ui: &mut egui::Ui, command_vec: &Vec<model::Variable>) {
        let mut value = String::new();
        for cmd in command_vec {
            value.push_str(cmd.get_expr());
            value.push('\n');
        }
        let mut text = value.as_str();
        ui.add(egui::TextEdit::multiline(&mut text).code_editor().desired_width(f32::INFINITY));
    }
}

impl eframe::App for GardenApp<'_> {
    fn update(&mut self, egui_ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let num_columns = 4;
        let row_height = 18.0;

        egui::CentralPanel::default().show(egui_ctx, |ui| {
            egui_ctx.set_pixels_per_point(2.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Query: ");
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::singleline(&mut self.query)
                            .hint_text("Tree query for the gardens, groups or trees to execute commands within")
                    );
                });

                // Global commands
                ui.separator();
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
                        button.context_menu(|ui| {
                            self.command_button_details(ui, command_vec);
                        });
                                               
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
                let contexts = query::resolve_trees(self.app_context, config, None, &self.query);
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
                                    ui.label(&tree_ctx.tree);
                                });
                                let tree = config.get_tree(&tree_ctx.tree);
                                if let Some(Ok(path)) = tree.map(|tree| tree.path_as_ref()) {
                                    row.col(|ui| {
                                        ui.label(path);
                                    });
                                } else {
                                    row.col(|ui| {
                                        ui.label("unknown");
                                    });
                                }
                            });
                        }
                    });
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
