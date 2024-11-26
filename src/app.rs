use bopler::extract_data_from_file;
use bopler::Patch;
use midir::MidiOutputConnection;
use midir::{MidiOutput, MidiOutputPort};
use regex::Regex;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    midi_out: MidiOutput,
    #[serde(skip)] // This how you opt-out of serialization of a field
    output_port: Option<MidiOutputPort>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        // Create a new MIDI output connection
        let midi_out = MidiOutput::new("My MIDI Output").expect("midi works");

        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            midi_out,
            output_port: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

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
            ui.heading("eframe template");

            // Get available ports
            let out_ports = self.midi_out.ports();

            // No ports available?
            if out_ports.is_empty() {
                println!("No MIDI output ports available!");
            }

            // List available ports
            for (i, p) in out_ports.iter().enumerate() {
                if ui
                    .button(format!(
                        "{}: {}",
                        i,
                        &self.midi_out.port_name(p).expect("port has name")
                    ))
                    .clicked()
                {
                    println!("\nOpening connection");
                    self.output_port = Some(p.clone());
                };
            }
            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            let patches = extract_data_from_file("all_patches.tsv");
            use egui_extras::{Column, TableBuilder};
            TableBuilder::new(ui)
                .column(Column::auto().resizable(true))
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.heading("Patch Name");
                    });
                    header.col(|ui| {
                        ui.heading("Category");
                    });
                    header.col(|ui| {
                        ui.heading("PC:MSB:LSB");
                    });
                })
                .body(|mut body| {
                    for patch in patches.expect("works").iter().filter(|p| {
                        p.category
                            .to_lowercase()
                            .contains(&self.label.to_lowercase())
                            || p.name.to_lowercase().contains(&self.label.to_lowercase())
                    }) {
                        body.row(25.0, |mut row| {
                            row.col(|ui| {
                                let click_but = format!("{}", patch.name);
                                let cb = ui.button(&click_but);
                                let midi_out =
                                    MidiOutput::new("My MIDI Output").expect("midi works");
                                if cb.clicked() {
                                    bopler::set_patch(
                                        patch,
                                        bopler::mpe::FULL_RANGE,
                                        midi_out
                                            .connect(&self.output_port.clone().unwrap(), "bopler")
                                            .as_mut()
                                            .unwrap(),
                                    );
                                }
                            });
                            row.col(|ui| {
                                ui.label(format!("{}", patch.category));
                            });
                            row.col(|ui| {
                                ui.label(format!("{}:{}:{}", patch.pc, patch.msb, patch.lsb));
                            });
                        });
                    }
                });

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

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
