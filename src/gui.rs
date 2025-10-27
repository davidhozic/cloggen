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
                ui.menu_button(RichText::new("Ukaz").font(FontId::proportional(24.0)), |ui| {
                    if ui.button(RichText::new("Ustvari mnenje").font(FontId::proportional(16.0))).clicked() {
                        self.state = UiState::NewReport;
                    };
                    if ui.button(RichText::new("Združi CSV").font(FontId::proportional(16.0))).clicked() {
                        self.state = UiState::MergeCsv;
                    };
                    if ui.button(RichText::new("Prevedi LaTeX").font(FontId::proportional(16.0))).clicked() {
                        self.state = UiState::CompileLatex;
                    };
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
                        )
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

#[derive(Default, PartialEq, Eq)]
enum UiState {
    #[default]
    NoCommand,
    NewReport,
    MergeCsv,
    CompileLatex
}
