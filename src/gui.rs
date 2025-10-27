//! Module defining Cloggen's Graphical User Interface.
use egui::{Color32, FontId, RichText};
use eframe::egui;


pub fn main_gui() {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Cloggen",
        options,
        Box::new(|_| Ok(Box::<Cloggen>::default()))
    ).unwrap();
}


#[derive(Default)]
struct Cloggen {
    state: UiState
}

impl eframe::App for Cloggen {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Toolbox
            ui.horizontal(|ui| {
                let current_selection: &str = self.state.clone().into();
                egui::ComboBox::from_id_salt("state").selected_text(current_selection).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.state, UiState::NewReport, UiState::NewReport.as_str());
                    ui.selectable_value(&mut self.state, UiState::MergeCsv, UiState::MergeCsv.as_str());
                    ui.selectable_value(&mut self.state, UiState::CompileLatex, UiState::CompileLatex.as_str());
                });
            });

            // Main content
            use UiState::*;
            match self.state {
                NoCommand => {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("Čakam ukaz").font(FontId::proportional(50.0)))
                    });
                }
                NewReport => {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Novo študentsko mnenje")
                                .font(FontId::proportional(16.0))
                        );
                        ui.centered_and_justified(|ui| {
                            
                        });
                    });
                }
                MergeCsv => {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Združevanje CSV podatkov iz STUDIS anket")
                                .font(FontId::proportional(16.0))
                        )
                    });
                }
                CompileLatex => {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Prevajanje LaTeX datoteke")
                                .font(FontId::proportional(16.0))
                        )
                    });
                }
            }
        });
    }
}


#[derive(Default, PartialEq, Eq, Clone)]
enum UiState {
    #[default]
    NoCommand,
    NewReport,
    MergeCsv,
    CompileLatex
}

impl UiState {
    fn as_str(self) -> &'static str {
        use UiState::*;
        match self {
            NoCommand => "Izberi ukaz",
            NewReport => "Novo poročilo",
            MergeCsv => "Združi CSV rezultate",
            CompileLatex => "Prevedi LaTeX",
        }
    }
}

impl Into<&str> for UiState {
    fn into(self) -> &'static str {
        self.as_str()
    }
}
