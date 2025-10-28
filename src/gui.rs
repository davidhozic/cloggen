//! Module defining Cloggen's Graphical User Interface.
use egui::{Color32, FontId, Frame, RichText, Stroke, ViewportBuilder};
use eframe::{egui};

use std::path::PathBuf;
use std::cell::Cell;


thread_local! {
    static PANIC_INFO: Cell<Option<String>> = const { Cell::new(None) };
}


pub fn main_gui() {
    // Forward panic to PANIC_INFO
    std::panic::set_hook(Box::new(|e| {
         PANIC_INFO.with(|trace| trace.set(Some(e.payload().downcast_ref::<String>().unwrap().to_string())));
    }));

    // Setup GUI
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_drag_and_drop(false),
        vsync: true,
        ..Default::default()
    };

    eframe::run_native(
        "Cloggen",
        options,
        Box::new(|_| Ok(Box::<Cloggen>::default()))
    ).unwrap();
}

#[derive(Default)]
struct Cloggen {
    menu: UiMenu,
    menu_state: UiMenuState
}

impl eframe::App for Cloggen {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Toolbox
            let menu_current = self.menu.clone();
            let current_selection: &str = self.menu.clone().into();
            egui::ComboBox::from_id_salt("state").selected_text(current_selection).show_ui(ui, |ui| {
                ui.selectable_value(&mut self.menu, UiMenu::NewReport, UiMenu::NewReport.as_str());
                ui.selectable_value(&mut self.menu, UiMenu::MergeCsv, UiMenu::MergeCsv.as_str());
                ui.selectable_value(&mut self.menu, UiMenu::CompileLatex, UiMenu::CompileLatex.as_str());
            });

            // Reinitialize the menu state
            if menu_current != self.menu {
                self.menu_state = self.menu.new_state();
            }

            // Main content
            match &mut self.menu_state {
                UiMenuState::NoCommand => {
                    ui.centered_and_justified(|ui| {
                        Frame::new().outer_margin(15.0).show(ui, |ui| {
                            ui.heading(RichText::new("CLogGen: Generator študentskih mnenj").font(FontId::proportional(50.0)))
                        })
                    });
                }
                UiMenuState::NewReport {
                    csv_file , responses_file, tex_template,
                    message, open_on_success
                } => {
                    ui.vertical_centered(|ui| {
                        ui.heading("Novo študentsko mnenje");

                        // Vhod
                        ui.add_space(10.0);
                        file_input(csv_file, ui, "STUDIS CSV", "csv");
                        file_input(responses_file, ui, "JSON nabor odzivov", "json");
                        file_input(tex_template, ui, "LaTeX predloga", "tex");

                        ui.add_space(50.0);
                        ui.vertical_centered(|ui| {
                                if ui.button(
                                    RichText::new("Ustvari in shrani")
                                        .font(FontId::proportional(24.0))
                                ).clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Študentsko mnenje", &["pdf", "tex"])
                                        .save_file()
                                    {
                                        match std::panic::catch_unwind(|| super::create::command_create(
                                            &csv_file,
                                            &responses_file,
                                            &tex_template,
                                            &super::config::create::SECTION_DEFAULT.to_string(),
                                            &super::config::create::FORMAT_DEFAULT,
                                            &Some(path)
                                        )) {
                                            Ok(filepath) => {
                                                *message = format!("Datoteka je bila shranjena: {filepath}");
                                                if *open_on_success {
                                                    let _ = open::that(filepath);
                                                }
                                            }
                                            Err(_) => {
                                                *message = format!(
                                                    "Napaka (nit je sprožila paniko!): {}",
                                                    PANIC_INFO.with(|p| p.take()).unwrap()
                                                );
                                            }
                                        };
                                    };
                                };
                            ui.checkbox(open_on_success, "Odpri ob uspehu");
                        });

                        // Status bottom
                        if message.len() > 0 {
                            ui.label(message.as_str());
                        }
                    });
                }
                // MergeCsv => {
                //     ui.vertical_centered(|ui| {
                //         ui.heading("Združevanje CSV podatkov iz STUDIS anket")
                //     });
                // }
                // CompileLatex => {
                //     ui.vertical_centered(|ui| {
                //         ui.heading("Prevajanje LaTeX datoteke")
                //     });
                // }
            }
        });
    }
}

fn file_input(file_var: &mut PathBuf, ui: &mut egui::Ui, heading: &str, extension: &str) {
    Frame::new()
        .stroke(Stroke::new(1.0, Color32::WHITE))
        .inner_margin(5.0).show(ui, |ui|
    {
        ui.heading(heading);
        ui.columns(2, |ui| {
            let button = ui[0].button("Izberi datoteko");
            if button.clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter(extension, &[extension]).pick_file() {
                    *file_var = path;
                }
            }

            let csv_file = file_var.as_os_str().to_str().unwrap();
            if csv_file.len() > 0 {
                ui[1].label(csv_file);
            }
        });
    });
}

#[derive(Default)]
enum UiMenuState {
    #[default]
    NoCommand,
    NewReport {
        csv_file: PathBuf,
        responses_file: PathBuf,
        tex_template: PathBuf,
        message: String,
        open_on_success: bool
    }
}

#[derive(Default, PartialEq, Eq, Clone)]
enum UiMenu {
    #[default]
    NoCommand,
    NewReport,
    MergeCsv,
    CompileLatex
}

impl UiMenu {
    fn as_str(&self) -> &'static str {
        use UiMenu::*;
        match self {
            NoCommand => "Izberi ukaz",
            NewReport => "Novo mnenje",
            MergeCsv => "Združi CSV rezultate",
            CompileLatex => "Prevedi LaTeX",
        }
    }

    fn new_state(&self) -> UiMenuState {
        use UiMenu::*;
        match self {
            NoCommand => UiMenuState::NoCommand,
            NewReport => UiMenuState::NewReport {
                csv_file: PathBuf::new(),
                responses_file: PathBuf::new(),
                tex_template: PathBuf::new(),
                message: String::new(),
                open_on_success: false
            },
            _ => todo!()
            // MergeCsv => "Združi CSV rezultate",
            // CompileLatex => "Prevedi LaTeX",
        }
    }
}

impl Into<&str> for UiMenu {
    fn into(self) -> &'static str {
        self.as_str()
    }
}
