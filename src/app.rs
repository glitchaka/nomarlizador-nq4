use std::path::PathBuf;

use eframe::egui;
use rfd::FileDialog;

use crate::{controller::MainController, model::NormalizationOptions};

pub struct Nq4App {
    controller: MainController,
    normalization: NormalizationOptions,
}

impl Nq4App {
    pub fn new(controller: MainController) -> Self {
        Self {
            controller,
            normalization: NormalizationOptions::default(),
        }
    }

    fn handle_drop(&mut self, ctx: &egui::Context) {
        let paths: Vec<PathBuf> = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect()
        });

        if !paths.is_empty() {
            self.controller.add_paths(paths);
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        if ui.button("Añadir MP3").clicked() {
            if let Some(files) = FileDialog::new()
                .add_filter("MP3", &["mp3"])
                .pick_files()
            {
                self.controller.add_paths(files);
            }
        }

        if ui.button("Añadir carpeta").clicked() {
            if let Some(folder) = FileDialog::new().pick_folder() {
                self.controller.add_folder(&folder);
            }
        }

        ui.separator();

        let has_selection = self.controller.selected().is_some();

        if ui
            .add_enabled(has_selection, egui::Button::new("Guardar tags"))
            .clicked()
        {
            self.controller.save_selected();
        }

        if ui
            .add_enabled(has_selection, egui::Button::new("Normalizar selección"))
            .clicked()
        {
            self.controller.normalize_selected(&self.normalization);
        }

        if ui
            .add_enabled(
                !self.controller.files().is_empty(),
                egui::Button::new("Normalizar todos"),
            )
            .clicked()
        {
            self.controller.normalize_all(&self.normalization);
        }
    }

    fn files_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("files")
            .resizable(true)
            .default_width(300.0)
            .min_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Archivos");
                ui.label(format!("{} MP3", self.controller.files().len()));
                ui.separator();

                let selected = self.controller.selected_index();
                let rows: Vec<(usize, String, String)> = self
                    .controller
                    .files()
                    .iter()
                    .enumerate()
                    .map(|(index, file)| {
                        (
                            index,
                            file.display_name(),
                            file.diagnostics.id3_version.clone(),
                        )
                    })
                    .collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, name, version) in rows {
                        let is_selected = selected == Some(index);
                        let response = ui.selectable_label(is_selected, name);
                        ui.small(version);
                        if response.clicked() {
                            self.controller.select(index);
                        }
                        ui.separator();
                    }
                });
            });
    }

    fn editor(&mut self, ui: &mut egui::Ui) {
        let mut change_cover = false;
        let mut remove_cover = false;

        {
            let Some(file) = self.controller.selected_mut() else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Normalizador NQ4");
                    ui.label("Añade MP3, una carpeta completa o arrastra archivos aquí.");
                    ui.add_space(8.0);
                    ui.label("El perfil iPod seguro modifica metadatos, no recodifica el audio.");
                });
                return;
            };

            ui.heading(file.display_name());
            ui.monospace(file.path.display().to_string());
            ui.add_space(10.0);

            egui::Grid::new("tag_editor")
                .num_columns(2)
                .spacing([18.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    Self::text_row(ui, "Título", &mut file.tags.title);
                    Self::text_row(ui, "Artista", &mut file.tags.artist);
                    Self::text_row(ui, "Álbum", &mut file.tags.album);
                    Self::text_row(ui, "Género", &mut file.tags.genre);
                    Self::text_row(ui, "Año", &mut file.tags.year);
                    Self::text_row(ui, "Pista", &mut file.tags.track);
                    Self::text_row(ui, "Disco", &mut file.tags.disc);

                    ui.label("Comentario");
                    ui.add(
                        egui::TextEdit::multiline(&mut file.tags.comment)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();
                });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let cover_label = file
                    .tags
                    .cover
                    .as_ref()
                    .map(|cover| {
                        format!(
                            "Portada: {} ({:.1} KiB)",
                            cover.mime_type,
                            cover.data.len() as f32 / 1024.0
                        )
                    })
                    .unwrap_or_else(|| "Sin portada".to_owned());

                ui.label(cover_label);
                change_cover = ui.button("Cambiar portada").clicked();
                remove_cover = file.tags.cover.is_some() && ui.button("Eliminar portada").clicked();
            });

            ui.separator();
            ui.heading("Diagnóstico");
            ui.label(format!("Versión: {}", file.diagnostics.id3_version));

            if let Some(cover) = &file.diagnostics.cover_description {
                ui.label(format!("Portada actual: {cover}"));
            }

            let warnings = file.diagnostics.warnings();
            if warnings.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(120, 200, 140), "Sin alertas ID3 detectadas.");
            } else {
                for warning in warnings {
                    ui.colored_label(egui::Color32::from_rgb(235, 185, 80), format!("• {warning}"));
                }
            }
        }

        if change_cover {
            if let Some(path) = FileDialog::new()
                .add_filter("Imagen", &["jpg", "jpeg", "png"])
                .pick_file()
            {
                self.controller.set_cover(&path);
            }
        }

        if remove_cover {
            self.controller.remove_cover();
        }
    }

    fn text_row(ui: &mut egui::Ui, label: &str, value: &mut String) {
        ui.label(label);
        ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
        ui.end_row();
    }

    fn options_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("options")
            .resizable(false)
            .default_width(255.0)
            .show(ctx, |ui| {
                ui.heading("Perfil iPod seguro");
                ui.label("Reescribe metadatos como ID3v2.3.");
                ui.separator();

                ui.checkbox(&mut self.normalization.create_backup, "Crear copia de seguridad");
                ui.checkbox(&mut self.normalization.strip_id3v1, "Eliminar ID3v1");
                ui.checkbox(&mut self.normalization.strip_apev2, "Eliminar APEv2");
                ui.checkbox(
                    &mut self.normalization.normalize_cover,
                    "Normalizar portada a JPEG",
                );

                ui.add_enabled_ui(self.normalization.normalize_cover, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Máx. portada");
                        ui.add(
                            egui::DragValue::new(&mut self.normalization.max_cover_size)
                                .range(128..=1200)
                                .suffix(" px"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Calidad JPEG");
                        ui.add(
                            egui::DragValue::new(&mut self.normalization.jpeg_quality)
                                .range(50..=100),
                        );
                    });
                });

                ui.separator();
                ui.small("TLEN siempre se elimina durante la normalización.");
                ui.small("El stream MP3 no se recodifica.");
                ui.small("Los backups quedan en .nq4-backup junto a los MP3.");
            });
    }
}

impl eframe::App for Nq4App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_drop(ctx);

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| self.toolbar(ui));
        });

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Estado:");
                ui.label(self.controller.status());
            });
        });

        self.files_panel(ctx);
        self.options_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.editor(ui));
        });
    }
}
