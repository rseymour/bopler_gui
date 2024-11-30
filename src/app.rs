use bopler::{extract_data_from_string, Patch};
use midir::{MidiOutput, MidiOutputPort};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    midi_out: MidiOutput,
    #[serde(skip)] // This how you opt-out of serialization of a field
    output_port: Option<MidiOutputPort>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    history: Vec<Patch>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    current: Option<Patch>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        // Create a new MIDI output connection
        let midi_out = MidiOutput::new("My MIDI Output").expect("midi works");

        Self {
            // Example stuff:
            label: "".to_owned(),
            midi_out,
            output_port: None,
            history: Vec::new(),
            current: None,
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
                ui.menu_button("History", |ui| {
                    for patch in self.history.iter() {
                        if ui.button(format!("{}", patch.name)).clicked() {
                            let midi_out = MidiOutput::new("My MIDI Output").expect("midi works");
                            let _ = bopler::set_patch(
                                patch,
                                bopler::mpe::FULL_RANGE,
                                midi_out
                                    .connect(&self.output_port.clone().unwrap(), "bopler")
                                    .as_mut()
                                    .expect("port exists"),
                            );
                            self.current = Some(patch.clone());
                        }
                    }
                });
                ui.add_space(16.0);
                if self.current.is_some() {
                    ui.menu_button(
                        format!("Current: {}", self.current.as_ref().unwrap().name),
                        |ui| {
                            if ui.button("Save").clicked() {
                                self.history.push(self.current.clone().unwrap());
                            }
                        },
                    );
                }
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Bopler JV-1010 MPE control");

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
                    self.output_port = Some(p.clone());
                };
            }
            ui.horizontal(|ui| {
                ui.label("Search patches: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.separator();

            let patch_string = std::include_str!("../all_patches.tsv");
            let patches = extract_data_from_string(&patch_string);
            use egui_extras::{Column, TableBuilder};
            TableBuilder::new(ui)
                .column(Column::auto_with_initial_suggestion(120.0).resizable(true))
                .column(Column::auto_with_initial_suggestion(160.0).resizable(true))
                .column(Column::remainder())
                .header(30.0, |mut header| {
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
                        let is_enabled = &self.output_port.is_some();
                        body.row(25.0, |mut row| {
                            row.col(|ui| {
                                let click_but = format!("{}", patch.name);
                                let cb = ui.add_enabled(*is_enabled, egui::Button::new(&click_but));
                                let midi_out =
                                    MidiOutput::new("My MIDI Output").expect("midi works");
                                if cb.clicked() {
                                    let _ = bopler::set_patch(
                                        patch,
                                        bopler::mpe::FULL_RANGE,
                                        midi_out
                                            .connect(&self.output_port.clone().unwrap(), "bopler")
                                            .as_mut()
                                            .expect("port exists"),
                                    );
                                    self.history.push(patch.clone());
                                    self.current = Some(patch.clone());
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
