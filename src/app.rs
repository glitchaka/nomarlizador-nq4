use std::{collections::VecDeque, path::PathBuf};

use eframe::egui;
use rfd::FileDialog;

use crate::{controller::MainController, model::NormalizationOptions};

struct BatchJob {
    queue: VecDeque<usize>,
    total: usize,
    completed: usize,
    errors: Vec<String>,
    options: NormalizationOptions,
}

impl BatchJob {
    fn progress(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.completed as f32 / self.total as f32
        }
    }
}

pub struct Nq4App {
    controller: MainController,
    normalization: NormalizationOptions,
    filter: String,
    show_files_panel: bool,
    show_options_panel: bool,
    show_diagnostics: bool,
    show_preferences: bool,
    show_about: bool,
    show_cover: bool,
    confirm_normalize_all: bool,
    batch: Option<BatchJob>,
    cover_texture_key: Option<String>,
    cover_texture: Option<egui::TextureHandle>,
}

impl Nq4App {
    pub fn new(controller: MainController) -> Self {
        Self {
            controller,
            normalization: NormalizationOptions::default(),
            filter: String::new(),
            show_files_panel: true,
            show_options_panel: true,
            show_diagnostics: false,
            show_preferences: false,
            show_about: false,
            show_cover: false,
            confirm_normalize_all: false,
            batch: None,
            cover_texture_key: None,
            cover_texture: None,
        }
    }

    fn open_files(&mut self) {
        if let Some(files) = FileDialog::new()
            .add_filter("MP3", &["mp3"])
            .pick_files()
        {
            self.controller.add_paths(files);
        }
    }

    fn open_folder(&mut self) {
        if let Some(folder) = FileDialog::new().pick_folder() {
            self.controller.add_folder(&folder);
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

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let ctrl = ctx.input(|input| input.modifiers.ctrl);
        let shift = ctx.input(|input| input.modifiers.shift);

        if ctrl && ctx.input(|input| input.key_pressed(egui::Key::O)) {
            self.open_files();
        }

        if ctrl && !shift && ctx.input(|input| input.key_pressed(egui::Key::S)) {
            self.controller.save_selected();
        }

        if ctrl && shift && ctx.input(|input| input.key_pressed(egui::Key::S)) {
            self.controller.save_all();
        }

        if ctx.input(|input| input.key_pressed(egui::Key::F5)) {
            self.controller.reload_selected();
        }

        if ctx.input(|input| input.key_pressed(egui::Key::Delete)) {
            self.controller.remove_selected();
        }
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Archivo", |ui| {
                    if ui.button("Abrir MP3…   Ctrl+O").clicked() {
                        self.open_files();
                    }
                    if ui.button("Abrir carpeta…").clicked() {
                        self.open_folder();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.controller.selected().is_some(),
                            egui::Button::new("Guardar archivo   Ctrl+S"),
                        )
                        .clicked()
                    {
                        self.controller.save_selected();
                    }
                    if ui
                        .add_enabled(
                            self.controller.dirty_count() > 0,
                            egui::Button::new("Guardar todo   Ctrl+Mayús+S"),
                        )
                        .clicked()
                    {
                        self.controller.save_all();
                    }
                    ui.separator();
                    if ui.button("Salir").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Editar", |ui| {
                    if ui
                        .add_enabled(
                            self.controller.selected().is_some(),
                            egui::Button::new("Descartar cambios / Recargar   F5"),
                        )
                        .clicked()
                    {
                        self.controller.reload_selected();
                    }
                    if ui
                        .add_enabled(
                            self.controller.selected().is_some(),
                            egui::Button::new("Quitar de la lista   Supr"),
                        )
                        .clicked()
                    {
                        self.controller.remove_selected();
                    }
                    if ui
                        .add_enabled(
                            !self.controller.files().is_empty(),
                            egui::Button::new("Vaciar lista"),
                        )
                        .clicked()
                    {
                        self.controller.clear();
                    }
                });

                ui.menu_button("Herramientas", |ui| {
                    if ui
                        .add_enabled(
                            self.controller.selected().is_some() && self.batch.is_none(),
                            egui::Button::new("Normalizar selección"),
                        )
                        .clicked()
                    {
                        self.controller.normalize_selected(&self.normalization);
                    }

                    if ui
                        .add_enabled(
                            !self.controller.files().is_empty() && self.batch.is_none(),
                            egui::Button::new("Normalizar todos…"),
                        )
                        .clicked()
                    {
                        self.confirm_normalize_all = true;
                    }

                    ui.separator();
                    if ui.button("Preferencias…").clicked() {
                        self.show_preferences = true;
                    }
                });

                ui.menu_button("Ver", |ui| {
                    ui.checkbox(&mut self.show_files_panel, "Panel de archivos");
                    ui.checkbox(&mut self.show_options_panel, "Perfil de normalización");
                    ui.checkbox(&mut self.show_diagnostics, "Diagnóstico detallado");

                    if ui
                        .add_enabled(
                            self.controller
                                .selected()
                                .and_then(|file| file.tags.cover.as_ref())
                                .is_some(),
                            egui::Button::new("Vista previa de portada"),
                        )
                        .clicked()
                    {
                        self.show_cover = true;
                    }
                });

                ui.menu_button("Ayuda", |ui| {
                    if ui.button("Acerca de Normalizador NQ4").clicked() {
                        self.show_about = true;
                    }
                });

                ui.separator();

                let dirty = self.controller.dirty_count();
                if dirty > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(235, 185, 80),
                        format!("{dirty} sin guardar"),
                    );
                }
            });
        });
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("＋ MP3").clicked() {
                    self.open_files();
                }

                if ui.button("＋ Carpeta").clicked() {
                    self.open_folder();
                }

                ui.separator();

                let has_selection = self.controller.selected().is_some();
                if ui
                    .add_enabled(has_selection, egui::Button::new("Guardar"))
                    .clicked()
                {
                    self.controller.save_selected();
                }

                if ui
                    .add_enabled(
                        has_selection && self.batch.is_none(),
                        egui::Button::new("Normalizar"),
                    )
                    .clicked()
                {
                    self.controller.normalize_selected(&self.normalization);
                }

                if ui
                    .add_enabled(
                        !self.controller.files().is_empty() && self.batch.is_none(),
                        egui::Button::new("Normalizar lote"),
                    )
                    .clicked()
                {
                    self.confirm_normalize_all = true;
                }

                ui.separator();

                ui.label("Buscar:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .desired_width(220.0)
                        .hint_text("archivo, título, artista, álbum"),
                );

                if !self.filter.is_empty() && ui.small_button("×").clicked() {
                    self.filter.clear();
                }
            });
        });
    }

    fn files_panel(&mut self, ctx: &egui::Context) {
        if !self.show_files_panel {
            return;
        }

        egui::SidePanel::left("files")
            .resizable(true)
            .default_width(330.0)
            .min_width(240.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Archivos");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{}", self.controller.files().len()));
                    });
                });

                ui.separator();

                let selected = self.controller.selected_index();
                let filter = self.filter.trim().to_lowercase();

                let rows: Vec<(usize, String, String, String, usize, bool)> = self
                    .controller
                    .files()
                    .iter()
                    .enumerate()
                    .filter_map(|(index, file)| {
                        let haystack = format!(
                            "{} {} {} {}",
                            file.display_name(),
                            file.tags.title,
                            file.tags.artist,
                            file.tags.album
                        )
                        .to_lowercase();

                        if !filter.is_empty() && !haystack.contains(&filter) {
                            return None;
                        }

                        Some((
                            index,
                            file.display_name(),
                            file.tags.artist.clone(),
                            file.diagnostics.id3_version.clone(),
                            file.diagnostics.warning_count(),
                            file.dirty,
                        ))
                    })
                    .collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, name, artist, version, warnings, dirty) in rows {
                        let is_selected = selected == Some(index);

                        let response = ui.selectable_label(
                            is_selected,
                            if dirty { format!("● {name}") } else { name },
                        );

                        ui.horizontal(|ui| {
                            if !artist.is_empty() {
                                ui.small(artist);
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if warnings > 0 {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(235, 185, 80),
                                        format!("⚠ {warnings}"),
                                    );
                                }
                                ui.small(version);
                            });
                        });

                        if response.clicked() {
                            self.controller.select(index);
                            self.cover_texture_key = None;
                        }

                        ui.separator();
                    }
                });
            });
    }

    fn editor(&mut self, ui: &mut egui::Ui) {
        let mut change_cover = false;
        let mut remove_cover = false;
        let mut show_cover = false;
        let mut changed = false;

        {
            let Some(file) = self.controller.selected_mut() else {
                ui.vertical_centered(|ui| {
                    ui.add_space(120.0);
                    ui.heading("Normalizador NQ4");
                    ui.label("Editor y normalizador ID3 portable.");
                    ui.add_space(8.0);
                    ui.label("Arrastra MP3 aquí o usa Archivo → Abrir MP3.");
                    ui.label("El perfil iPod seguro nunca recodifica el audio.");
                });
                return;
            };

            ui.horizontal(|ui| {
                ui.heading(file.display_name());
                if file.dirty {
                    ui.colored_label(
                        egui::Color32::from_rgb(235, 185, 80),
                        "● cambios sin guardar",
                    );
                }
            });

            ui.monospace(file.path.display().to_string());
            ui.add_space(10.0);

            egui::Grid::new("tag_editor")
                .num_columns(2)
                .spacing([20.0, 10.0])
                .striped(true)
                .show(ui, |ui| {
                    changed |= Self::text_row(ui, "Título", &mut file.tags.title);
                    changed |= Self::text_row(ui, "Artista", &mut file.tags.artist);
                    changed |= Self::text_row(ui, "Álbum", &mut file.tags.album);
                    changed |= Self::text_row(ui, "Género", &mut file.tags.genre);
                    changed |= Self::text_row(ui, "Año", &mut file.tags.year);
                    changed |= Self::text_row(ui, "Pista", &mut file.tags.track);
                    changed |= Self::text_row(ui, "Disco", &mut file.tags.disc);

                    ui.label("Comentario");
                    changed |= ui
                        .add(
                            egui::TextEdit::multiline(&mut file.tags.comment)
                                .desired_rows(4)
                                .desired_width(f32::INFINITY),
                        )
                        .changed();
                    ui.end_row();
                });

            ui.add_space(14.0);
            ui.group(|ui| {
                ui.horizontal_wrapped(|ui| {
                    let cover_label = file
                        .tags
                        .cover
                        .as_ref()
                        .map(|cover| {
                            format!(
                                "Portada: {} · {:.1} KiB",
                                cover.mime_type,
                                cover.data.len() as f32 / 1024.0
                            )
                        })
                        .unwrap_or_else(|| "Sin portada".to_owned());

                    ui.label(cover_label);
                    change_cover = ui.button("Cambiar…").clicked();
                    remove_cover =
                        file.tags.cover.is_some() && ui.button("Eliminar").clicked();
                    show_cover =
                        file.tags.cover.is_some() && ui.button("Vista previa").clicked();
                });
            });

            if changed {
                file.dirty = true;
            }

            ui.add_space(12.0);
            ui.separator();
            ui.heading("Resumen técnico");

            egui::Grid::new("summary")
                .num_columns(2)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("ID3");
                    ui.label(&file.diagnostics.id3_version);
                    ui.end_row();

                    ui.label("TLEN");
                    ui.label(
                        file.diagnostics
                            .raw_tlen
                            .as_deref()
                            .unwrap_or("No presente"),
                    );
                    ui.end_row();

                    ui.label("ID3v1");
                    ui.label(if file.diagnostics.has_id3v1 { "Sí" } else { "No" });
                    ui.end_row();

                    ui.label("APEv2");
                    ui.label(if file.diagnostics.has_apev2 { "Sí" } else { "No" });
                    ui.end_row();
                });

            let warnings = file.diagnostics.warnings();
            if warnings.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(120, 200, 140),
                    "Sin alertas ID3 detectadas.",
                );
            } else {
                ui.add_space(6.0);
                for warning in warnings {
                    ui.colored_label(
                        egui::Color32::from_rgb(235, 185, 80),
                        format!("⚠ {warning}"),
                    );
                }
            }
        }

        if change_cover {
            if let Some(path) = FileDialog::new()
                .add_filter("Imagen", &["jpg", "jpeg", "png"])
                .pick_file()
            {
                self.controller.set_cover(&path);
                self.cover_texture_key = None;
            }
        }

        if remove_cover {
            self.controller.remove_cover();
            self.cover_texture_key = None;
        }

        if show_cover {
            self.show_cover = true;
            self.cover_texture_key = None;
        }
    }

    fn text_row(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
        ui.label(label);
        let changed = ui
            .add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY))
            .changed();
        ui.end_row();
        changed
    }

    fn options_panel(&mut self, ctx: &egui::Context) {
        if !self.show_options_panel {
            return;
        }

        egui::SidePanel::right("options")
            .resizable(false)
            .default_width(275.0)
            .show(ctx, |ui| {
                ui.heading("Perfil iPod seguro");
                ui.label("Normalización conservadora para reproductores antiguos.");
                ui.separator();

                ui.checkbox(
                    &mut self.normalization.create_backup,
                    "Crear copia de seguridad",
                );
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
                ui.label("La normalización:");
                ui.small("• escribe ID3v2.3;");
                ui.small("• elimina TLEN;");
                ui.small("• conserva título, artista, álbum, año, pista y comentario;");
                ui.small("• no transcodifica el stream MP3.");

                ui.separator();

                let warnings = self.controller.warning_count();
                if warnings > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(235, 185, 80),
                        format!("⚠ {warnings} alerta(s) en la lista"),
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(120, 200, 140),
                        "Sin alertas en la lista",
                    );
                }
            });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            if let Some(batch) = &self.batch {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Normalizando {}/{}",
                        batch.completed, batch.total
                    ));
                    ui.add(
                        egui::ProgressBar::new(batch.progress())
                            .desired_width(220.0)
                            .show_percentage(),
                    );
                    if !batch.errors.is_empty() {
                        ui.colored_label(
                            egui::Color32::from_rgb(225, 110, 95),
                            format!("{} error(es)", batch.errors.len()),
                        );
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Estado:");
                    ui.label(self.controller.status());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!(
                            "{} archivo(s) · {} sin guardar",
                            self.controller.files().len(),
                            self.controller.dirty_count()
                        ));
                    });
                });
            }
        });
    }

    fn confirm_normalize_all_window(&mut self, ctx: &egui::Context) {
        if !self.confirm_normalize_all {
            return;
        }

        egui::Window::new("Normalizar por lote")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!(
                    "Se normalizarán {} archivo(s) con el perfil actual.",
                    self.controller.files().len()
                ));

                if self.normalization.create_backup {
                    ui.label("Se creará una copia de seguridad antes de cada modificación.");
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(235, 185, 80),
                        "La copia de seguridad está desactivada.",
                    );
                }

                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancelar").clicked() {
                        self.confirm_normalize_all = false;
                    }

                    if ui.button("Normalizar todos").clicked() {
                        let total = self.controller.files().len();
                        self.batch = Some(BatchJob {
                            queue: (0..total).collect(),
                            total,
                            completed: 0,
                            errors: Vec::new(),
                            options: self.normalization.clone(),
                        });
                        self.confirm_normalize_all = false;
                    }
                });
            });
    }

    fn preferences_window(&mut self, ctx: &egui::Context) {
        if !self.show_preferences {
            return;
        }

        egui::Window::new("Preferencias")
            .open(&mut self.show_preferences)
            .default_width(460.0)
            .show(ctx, |ui| {
                ui.heading("Normalización");
                ui.checkbox(
                    &mut self.normalization.create_backup,
                    "Crear backup antes de modificar",
                );
                ui.checkbox(
                    &mut self.normalization.strip_id3v1,
                    "Eliminar etiqueta ID3v1 residual",
                );
                ui.checkbox(
                    &mut self.normalization.strip_apev2,
                    "Eliminar etiqueta APEv2 residual",
                );
                ui.checkbox(
                    &mut self.normalization.normalize_cover,
                    "Convertir portada a JPEG compatible",
                );

                ui.separator();
                ui.label("Compatibilidad de portada");

                ui.horizontal(|ui| {
                    ui.label("Tamaño máximo:");
                    ui.add(
                        egui::DragValue::new(&mut self.normalization.max_cover_size)
                            .range(128..=1200)
                            .suffix(" px"),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Calidad JPEG:");
                    ui.add(
                        egui::DragValue::new(&mut self.normalization.jpeg_quality)
                            .range(50..=100),
                    );
                });

                ui.separator();
                ui.small("Las preferencias se aplican a la sesión actual.");
            });
    }

    fn diagnostics_window(&mut self, ctx: &egui::Context) {
        if !self.show_diagnostics {
            return;
        }

        egui::Window::new("Diagnóstico ID3")
            .open(&mut self.show_diagnostics)
            .default_width(520.0)
            .show(ctx, |ui| {
                let Some(file) = self.controller.selected() else {
                    ui.label("Selecciona un archivo.");
                    return;
                };

                ui.heading(file.display_name());
                ui.monospace(file.path.display().to_string());
                ui.separator();

                ui.label(format!("Versión ID3v2: {}", file.diagnostics.id3_version));
                ui.label(format!(
                    "TLEN: {}",
                    file.diagnostics
                        .raw_tlen
                        .as_deref()
                        .unwrap_or("no presente")
                ));
                ui.label(format!(
                    "ID3v1: {}",
                    if file.diagnostics.has_id3v1 { "presente" } else { "no" }
                ));
                ui.label(format!(
                    "APEv2: {}",
                    if file.diagnostics.has_apev2 { "presente" } else { "no" }
                ));

                if let Some(cover) = &file.diagnostics.cover_description {
                    ui.label(format!("Portada: {cover}"));
                } else {
                    ui.label("Portada: no presente");
                }

                ui.separator();
                let warnings = file.diagnostics.warnings();
                if warnings.is_empty() {
                    ui.label("No se detectaron incompatibilidades evidentes.");
                } else {
                    for warning in warnings {
                        ui.colored_label(
                            egui::Color32::from_rgb(235, 185, 80),
                            format!("⚠ {warning}"),
                        );
                    }
                }
            });
    }

    fn cover_window(&mut self, ctx: &egui::Context) {
        if !self.show_cover {
            return;
        }

        let Some(file) = self.controller.selected() else {
            self.show_cover = false;
            return;
        };

        let Some(cover) = &file.tags.cover else {
            self.show_cover = false;
            return;
        };

        let key = format!("{}:{}", file.path.display(), cover.data.len());
        if self.cover_texture_key.as_deref() != Some(&key) {
            self.cover_texture = image::load_from_memory(&cover.data)
                .ok()
                .map(|image| {
                    let rgba = image.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = rgba.into_raw();
                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    ctx.load_texture("cover_preview", color_image, egui::TextureOptions::LINEAR)
                });
            self.cover_texture_key = Some(key);
        }

        egui::Window::new("Vista previa de portada")
            .open(&mut self.show_cover)
            .resizable(true)
            .default_size([520.0, 560.0])
            .show(ctx, |ui| {
                if let Some(texture) = &self.cover_texture {
                    let available = ui.available_size();
                    let source = texture.size_vec2();
                    let scale = (available.x / source.x)
                        .min((available.y - 50.0).max(100.0) / source.y)
                        .min(1.0);
                    let size = source * scale;
                    ui.add(egui::Image::new((texture.id(), size)));
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(225, 110, 95),
                        "No se pudo decodificar la portada.",
                    );
                }

                ui.separator();
                ui.label(format!(
                    "{} · {:.1} KiB",
                    cover.mime_type,
                    cover.data.len() as f32 / 1024.0
                ));
            });
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }

        egui::Window::new("Acerca de")
            .open(&mut self.show_about)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Normalizador NQ4");
                ui.label("Editor y normalizador portable de etiquetas ID3.");
                ui.add_space(8.0);
                ui.label("Rust · MVC · SOLID");
                ui.label("Perfil de compatibilidad para iPod y reproductores antiguos.");
                ui.add_space(8.0);
                ui.small("El audio MP3 nunca se recodifica durante la normalización.");
            });
    }

    fn process_batch_step(&mut self, ctx: &egui::Context) {
        let Some(mut batch) = self.batch.take() else {
            return;
        };

        if let Some(index) = batch.queue.pop_front() {
            if let Err(error) = self.controller.normalize_one(index, &batch.options) {
                let name = self
                    .controller
                    .files()
                    .get(index)
                    .map(|file| file.display_name())
                    .unwrap_or_else(|| format!("#{index}"));
                batch.errors.push(format!("{name}: {error}"));
            }

            batch.completed += 1;
            self.batch = Some(batch);
            ctx.request_repaint();
            return;
        }

        if batch.errors.is_empty() {
            self.controller.set_status(format!(
                "{} archivo(s) normalizado(s) correctamente.",
                batch.total
            ));
        } else {
            self.controller.set_status(format!(
                "{} completado(s), {} error(es). {}",
                batch.total.saturating_sub(batch.errors.len()),
                batch.errors.len(),
                batch.errors[0]
            ));
        }
    }
}

impl eframe::App for Nq4App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_drop(ctx);
        self.handle_shortcuts(ctx);
        self.process_batch_step(ctx);

        self.menu_bar(ctx);
        self.toolbar(ctx);
        self.status_bar(ctx);
        self.files_panel(ctx);
        self.options_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.editor(ui));
        });

        self.confirm_normalize_all_window(ctx);
        self.preferences_window(ctx);
        self.diagnostics_window(ctx);
        self.cover_window(ctx);
        self.about_window(ctx);
    }
}
