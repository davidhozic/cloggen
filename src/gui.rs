//! Module defining Cloggen's Graphical User Interface.
use egui::{Color32, FontId, Frame, RichText, Stroke, ViewportBuilder};
use eframe::{egui};

use std::{ops::BitAnd, path::PathBuf};


pub fn main_gui() {
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
                                        match super::create::command_create(
                                            &csv_file,
                                            &responses_file,
                                            &tex_template,
                                            &super::config::create::SECTION_DEFAULT.to_string(),
                                            &super::config::create::FORMAT_DEFAULT,
                                            &Some(path)
                                        ) {
                                            Ok(filepath) => {
                                                *message = format!("Datoteka je bila shranjena: {filepath}");
                                                if *open_on_success {
                                                    let _ = open::that(filepath);
                                                }
                                            }
                                            Err(err) => {
                                                *message = format!("Napaka: {}", err);
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
                UiMenuState::MergeCsv { csv_files, selected_files, message } => {
                    ui.vertical_centered(|ui| {
                        ui.heading("Združevanje CSV podatkov iz STUDIS anket");
                        
                        // Control panel
                        egui::TopBottomPanel::top("top").show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Dodaj datoteke").clicked() {
                                    if let Some(files) = rfd::FileDialog::new().add_filter("Vhodni CSV", &["csv"]).pick_files() {
                                        csv_files.extend(files);
                                    }
                                }
                                if ui.button("Odstrani izbiro").clicked() {
                                    let mut new_files = Vec::new();
                                    let mut n_deleted = 0;
                                    for (i, file) in csv_files.iter().enumerate() {
                                        if selected_files.bitand(1 << i) == 0 {
                                            new_files.push(file.clone());
                                        }
                                        else {
                                            n_deleted += 1;
                                        }

                                        if csv_files.len() - n_deleted <= i {
                                            *selected_files &= !(1 << i);
                                        }
                                    }
                                    *csv_files = new_files;
                                }

                                const MERGE_BNT_TEXT: &str = "Združi vse datoteke";
                                if csv_files.len() > 1 {  // Needs at least two files to merge
                                    if ui.button(MERGE_BNT_TEXT).clicked() {
                                        if let Some(file) = rfd::FileDialog::new().add_filter("Združen CSV", &["csv"]).save_file() {
                                            super::merge::command_merge(&csv_files, &super::config::merge::SECTION_DEFAULT, &file);
                                            *message = format!("Datoteka je shranjena: {}", file.display());
                                        };
                                    };   
                                }
                                else {
                                    ui.button(RichText::new(MERGE_BNT_TEXT).weak()).on_hover_cursor(
                                        egui::CursorIcon::NotAllowed
                                    ).on_hover_text("Za združevanje sta potrebni vsaj dve datoteki.");
                                }
                            });

                            // Message after operation
                            if message.len() > 0 {
                                ui.label(message.as_str());
                            }
                        });

                        // Added files listbox
                        egui::ScrollArea::vertical().show(ui, |ui|
                        {
                            for (i, file) in csv_files.iter().enumerate() {
                                let selected = selected_files.bitand(1 << i) > 0;
                                if ui.selectable_label(selected, file.to_str().unwrap()).clicked() {
                                    if selected {
                                        *selected_files &= !(1 << i);
                                    }
                                    else {
                                        *selected_files |= 1 << i;
                                    }
                                };
                            }
                        });
                    });
                }
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
    },
    MergeCsv {
        csv_files: Vec<PathBuf>,
        selected_files: u32,
        message: String,
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
            MergeCsv => UiMenuState::MergeCsv { csv_files: vec![], selected_files: 0, message: String::new() },
            _ => todo!()
            // CompileLatex => "Prevedi LaTeX",
        }
    }
}

impl Into<&str> for UiMenu {
    fn into(self) -> &'static str {
        self.as_str()
    }
}
