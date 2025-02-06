use chrono::prelude::*;

use std::{path::PathBuf, sync::Arc};

use egui::{
    debug_text::print, style::Selection, Button, Color32, ComboBox, Grid, Label, RichText,
    Rounding, Slider, Stroke, Vec2, Visuals,
};
use rusqlite::Connection;

use egui_extras::DatePickerButton;
use egui_phosphor;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    db_path: PathBuf,
    export_path: PathBuf,
    start_date: String,
    end_date: String,
    table_name: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    error_message: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            db_path: std::env::current_dir().unwrap().with_file_name("Logger.db"),
            export_path: std::env::current_dir()
                .unwrap()
                .with_file_name("export.csv"),
            label: "Hello World!".to_owned(),
            table_name: "S7_1200".to_string(),
            start_date: "01/01/2025".to_string(),
            end_date: "01/01/2025".to_string(),
            value: 2.7,
            error_message: "".to_string(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "custom_font".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/assets/plex.ttf"
            ))),
            //egui::FontData::from_static(include_bytes!("../assets/dejavu.ttf")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "custom_font".to_owned());

        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::variants::Variant::Regular);

        //egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_fonts(fonts);

        // Configuring visuals.

        let mut visuals = Visuals::light();
        visuals.selection = Selection {
            bg_fill: Color32::from_rgb(81, 129, 154),
            stroke: Stroke::new(1.0, Color32::WHITE),
        };

        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(180, 180, 180);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(180, 180, 180);
        visuals.widgets.inactive.rounding = Rounding::ZERO;
        visuals.widgets.noninteractive.rounding = Rounding::ZERO;
        visuals.widgets.active.rounding = Rounding::ZERO;
        visuals.widgets.hovered.rounding = Rounding::ZERO;
        visuals.window_rounding = Rounding::ZERO;
        visuals.window_fill = Color32::from_rgb(197, 197, 197);
        visuals.menu_rounding = Rounding::ZERO;
        visuals.panel_fill = Color32::from_rgb(200, 200, 200);
        visuals.striped = true;
        visuals.slider_trailing_fill = true;

        cc.egui_ctx.set_visuals(visuals);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Log Export");

            // let date = chrono::offset::Local::now().date_naive();
            // let formatted_date = date.format("%d/%m/%Y").to_string();
            // self.end_date = formatted_date;

            egui::Grid::new("hello")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Start time");
                    ui.text_edit_singleline(&mut self.start_date);
                    ui.end_row();
                    ui.label("End time");
                    ui.text_edit_singleline(&mut self.end_date);
                    ui.end_row();
                    ui.label("Database path");
                    let db_path_name = std::fs::canonicalize(&self.db_path).unwrap();
                    ui.label(format!("{:?}", &db_path_name.clone().into_os_string()));
                    ui.end_row();
                    ui.label("Table");
                    ui.text_edit_singleline(&mut self.table_name);
                    ui.end_row();
                    if ui.button("Select DB").clicked() {
                        let res = rfd::FileDialog::new()
                            .add_filter("database", &["db", "sqlite"])
                            .set_directory(&db_path_name.parent().unwrap())
                            .pick_file();

                        if let Some(path) = res {
                            self.db_path = path;
                        }
                    }
                    if ui.button("Export").clicked() {
                        let res = rfd::FileDialog::new()
                            .add_filter("TXT", &["txt", "csv"])
                            .set_directory(&db_path_name.parent().unwrap())
                            .save_file();

                        if let Some(path) = res {
                            self.export_path = path;
                            let conn = Connection::open(&self.db_path);
                            if let Ok(conn) = conn {
                                let start_time = NaiveDateTime::parse_from_str(
                                    &self.start_date,
                                    "%d/%m/%Y %H:%M:%S",
                                );
                                let end_time = NaiveDateTime::parse_from_str(
                                    &self.end_date,
                                    "%d/%m/%Y %H:%M:%S",
                                );

                                if start_time.is_ok() && end_time.is_ok() {
                                    self.error_message = "".to_owned();
                                    let start_timestamp = start_time.unwrap().and_utc().timestamp();
                                    let end_timestamp = end_time.unwrap().and_utc().timestamp();

                                    println!("{}", start_timestamp);
                                    println!("{}", end_timestamp);
                                    let res = conn.prepare(
                                        format!(
                                            "
                                            SELECT * FROM {}
                                            WHERE timestamp BETWEEN ?1 AND ?2
                                        ",
                                            &self.table_name
                                        )
                                        .as_str(),
                                    );

                                    if let Ok(mut res) = res {
                                        let rows = res.query([start_timestamp, end_timestamp]);

                                        //let rows = res.query_and_then(
                                        //[start_timestamp, end_timestamp],
                                        //|row| row.get::<_, usize>(1),
                                        //);

                                        if let Ok(mut rows) = rows {
                                            while let Some(row) = rows.next().unwrap() {
                                                let res_timestamp = row.get::<_, usize>(1).unwrap();
                                                let res_tag = row.get::<_, String>(2).unwrap();
                                                println!("{} .  {}", res_timestamp, res_tag);
                                            }
                                        }
                                    } else {
                                        println!("ERROR");
                                    }
                                } else {
                                    self.error_message =
                                        "Could not read the time and date values.".to_string();
                                }
                            }
                        }
                    }
                    ui.end_row();
                    ui.label("Status");
                    ui.label(&self.error_message);
                    ui.end_row();
                });
            ui.separator();

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
