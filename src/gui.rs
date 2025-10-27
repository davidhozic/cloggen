//! Module defining Cloggen's Graphical User Interface.
use egui::{Color32, FontId, Frame, RichText, Stroke, ViewportBuilder};
use eframe::{egui};

use std::path::PathBuf;


pub fn main_gui() {
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
                UiMenuState::NewReport { csv_file , responses_file, tex_template } => {
                    ui.vertical_centered(|ui| {
                        ui.heading("Novo študentsko mnenje");

                        ui.add_space(10.0);
                        file_input(csv_file, ui, "STUDIS CSV", "csv");
                        file_input(responses_file, ui, "JSON nabor odzivov", "json");
                        file_input(tex_template, ui, "LaTeX predloga", "tex");
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
        tex_template: PathBuf
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
            NewReport => UiMenuState::NewReport { csv_file: PathBuf::new(), responses_file: PathBuf::new(), tex_template: PathBuf::new() },
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
