use chrono::prelude::*;

use std::{io::Write, path::PathBuf, str::FromStr, sync::Arc};

use egui::{
    debug_text::print, epaint::tessellator::Path, style::Selection, Button, Color32, ComboBox,
    Grid, Label, RichText, Rounding, Slider, Stroke, Vec2, Visuals,
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
    #[serde(skip)] // This how you opt-out of serialization of a field
    db_path: PathBuf,
    #[serde(skip)] // This how you opt-out of serialization of a field
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
            //db_path: std::env::current_dir().unwrap().with_file_name("Logger.db"),
            db_path: PathBuf::from_str("./Logger.db").unwrap(),
            export_path: std::env::current_dir()
                .unwrap()
                .with_file_name("export.csv"),
            label: "Hello World!".to_owned(),
            table_name: "S7_1200".to_string(),
            start_date: "01/01/2025 00:00:00".to_string(),
            end_date: "01/01/2025 00:00:00".to_string(),
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
                    ui.label(format!(
                        "{}",
                        &db_path_name.clone().into_os_string().into_string().unwrap()
                    ));
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
                            .add_filter("csv", &["csv"])
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
                                            let mut row_count = 0;
                                            let mut current_timestamp = 0;
                                            let file = std::fs::File::options()
                                                .create(true)
                                                .append(true)
                                                .open(&self.export_path);
                                            if let Ok(mut file) = file {
                                                let mut line_string = String::new();
                                                while let Some(row) = rows.next().unwrap() {
                                                    let res_id = row.get::<_, usize>(0).unwrap();
                                                    let res_timestamp =
                                                        row.get::<_, usize>(1).unwrap();
                                                    // if is_new_line {

                                                    //  }

                                                    if res_timestamp == current_timestamp {
                                                        
                                                        let res_value = row.get::<_, f32>(4).unwrap();
                                                        line_string.push_str(format!("{},", res_value).as_str());
                                                    } else {

                                                        let line = format!("{}\r\n", line_string);
                                                        if let Ok(_write_all) =
                                                            file.write_all(&line.as_bytes())
                                                        {
                                                            row_count += 1;
                                                        } else {
                                                            self.error_message = format!(
                                                                "Could not write to file: {:?}",
                                                                &self.export_path
                                                            );
                                                        }
                                                        line_string.clear();
                                                        line_string.push_str(format!("{},", row_count).as_str());

                                                        line_string.push_str(format!("{},", res_timestamp).as_str());
                                                        let timestamp = Utc
                                                            .timestamp_opt(res_timestamp as i64, 0)
                                                            .unwrap()
                                                            .format("%d/%m/%y,%H:%M:%S,")
                                                            .to_string();
                                                        line_string.push_str(timestamp.as_str());
                                                        let res_value = row.get::<_, f32>(4).unwrap();
                                                        line_string.push_str(format!("{},", res_value).as_str());
                                                        current_timestamp = res_timestamp;
                                                        // is_new_line = false;
                                                    }


                                                    // let res_tag = row.get::<_, String>(2).unwrap();
                                                    // let res_desc = row.get::<_, String>(3).unwrap();
                                                    // let res_value = row.get::<_, f32>(4).unwrap();
                                                    // let line = format!(
                                                    //     "{},{},{},{},{},{}\n",
                                                    //     res_id,
                                                    //     res_timestamp,
                                                    //     timestamp,
                                                    //     res_tag,
                                                    //     res_desc,
                                                    //     res_value
                                                    // );

                                                    // if let Ok(_write_all) =
                                                    //     file.write_all(&line.as_bytes())
                                                    // {
                                                    //     row_count += 1;
                                                    // } else {
                                                    //     self.error_message = format!(
                                                    //         "Could not write to file: {:?}",
                                                    //         &self.export_path
                                                    //     );
                                                    // }
                                                }
                                            } else {
                                                self.error_message = format!(
                                                    "Could not open or create file: {:?}",
                                                    &self.export_path
                                                );
                                            }
                                            self.error_message =
                                                format!("{} rows extracted.", row_count);
                                        } else {
                                            self.error_message =
                                                "Error reading database rows.".to_owned();
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
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
    });
}
