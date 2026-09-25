use std::{collections::VecDeque, path::PathBuf};

use eframe::egui;
use rfd::FileDialog;

use crate::{
    controller::MainController,
    model::{NormalizationOptions, NORMALIZED_FOLDER_NAME},
};

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
    show_editor: bool,
    show_diagnostics: bool,
    show_preferences: bool,
    show_about: bool,
    show_cover: bool,
    confirm_normalize_all: bool,
    batch: Option<BatchJob>,
    cover_texture_key: Option<String>,
    cover_texture: Option<egui::TextureHandle>,
    style_configured: bool,
}

impl Nq4App {
    pub fn new(controller: MainController) -> Self {
        Self {
            controller,
            normalization: NormalizationOptions::default(),
            filter: String::new(),
            show_editor: false,
            show_diagnostics: false,
            show_preferences: false,
            show_about: false,
            show_cover: false,
            confirm_normalize_all: false,
            batch: None,
            cover_texture_key: None,
            cover_texture: None,
            style_configured: false,
        }
    }

    fn configure_style(&mut self, ctx: &egui::Context) {
        if self.style_configured {
            return;
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.interact_size.y = 28.0;
        ctx.set_style(style);
        self.style_configured = true;
    }

    fn open_files(&mut self) {
        if let Some(files) = FileDialog::new().add_filter("MP3", &["mp3"]).pick_files() {
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

        if ctrl && ctx.input(|input| input.key_pressed(egui::Key::E)) {
            self.show_editor = self.controller.selected().is_some();
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

        if ctx.input(|input| input.key_pressed(egui::Key::Delete)) && !self.show_editor {
            self.controller.remove_selected();
        }
    }

    fn title_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("custom_title_bar")
            .exact_height(36.0)
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(23, 26, 31)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let controls_width = 80.0;
                    let drag_width = (ui.available_width() - controls_width).max(100.0);
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(drag_width, 30.0),
                        egui::Sense::click_and_drag(),
                    );

                    ui.painter().text(
                        rect.left_center() + egui::vec2(10.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        "NQ4  ·  Normalizador de MP3",
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_rgb(232, 236, 240),
                    );

                    if response.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }

                    if ui
                        .add_sized([34.0, 28.0], egui::Button::new("—").frame(false))
                        .on_hover_text("Minimizar")
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }

                    if ui
                        .add_sized([34.0, 28.0], egui::Button::new("×").frame(false))
                        .on_hover_text("Cerrar")
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar")
            .exact_height(34.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.menu_button("Archivo", |ui| {
                        if ui.button("Abrir MP3…   Ctrl+O").clicked() {
                            self.open_files();
                            ui.close();
                        }
                        if ui.button("Abrir carpeta…").clicked() {
                            self.open_folder();
                            ui.close();
                        }
                        ui.separator();
                        if ui
                            .add_enabled(
                                self.controller.dirty_count() > 0,
                                egui::Button::new("Guardar cambios   Ctrl+S"),
                            )
                            .clicked()
                        {
                            self.controller.save_selected();
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                self.controller.dirty_count() > 0,
                                egui::Button::new("Guardar todos   Ctrl+Mayús+S"),
                            )
                            .clicked()
                        {
                            self.controller.save_all();
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                self.controller.last_output_dir().is_some(),
                                egui::Button::new("Abrir carpeta de salida"),
                            )
                            .clicked()
                        {
                            self.controller.open_output_folder();
                            ui.close();
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
                                egui::Button::new("Editar tags…   Ctrl+E"),
                            )
                            .clicked()
                        {
                            self.show_editor = true;
                            ui.close();
                        }

                        if ui
                            .add_enabled(
                                self.controller.selected().is_some(),
                                egui::Button::new("Descartar cambios / Recargar   F5"),
                            )
                            .clicked()
                        {
                            self.controller.reload_selected();
                            ui.close();
                        }

                        ui.separator();

                        if ui
                            .add_enabled(
                                self.controller.selected().is_some(),
                                egui::Button::new("Quitar de la lista   Supr"),
                            )
                            .clicked()
                        {
                            self.controller.remove_selected();
                            ui.close();
                        }

                        if ui
                            .add_enabled(
                                !self.controller.files().is_empty(),
                                egui::Button::new("Vaciar lista"),
                            )
                            .clicked()
                        {
                            self.controller.clear();
                            ui.close();
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
                            ui.close();
                        }

                        if ui
                            .add_enabled(
                                !self.controller.files().is_empty() && self.batch.is_none(),
                                egui::Button::new("Normalizar todos…"),
                            )
                            .clicked()
                        {
                            self.confirm_normalize_all = true;
                            ui.close();
                        }

                        ui.separator();

                        if ui.button("Preferencias…").clicked() {
                            self.show_preferences = true;
                            ui.close();
                        }
                    });

                    ui.menu_button("Ver", |ui| {
                        if ui
                            .add_enabled(
                                self.controller.selected().is_some(),
                                egui::Button::new("Diagnóstico ID3"),
                            )
                            .clicked()
                        {
                            self.show_diagnostics = true;
                            ui.close();
                        }

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
                            ui.close();
                        }
                    });

                    ui.menu_button("Ayuda", |ui| {
                        if ui.button("Acerca de Normalizador NQ4").clicked() {
                            self.show_about = true;
                            ui.close();
                        }
                    });

                    let dirty = self.controller.dirty_count();
                    if dirty > 0 {
                        ui.separator();
                        ui.colored_label(
                            egui::Color32::from_rgb(235, 185, 80),
                            format!("{dirty} sin guardar"),
                        );
                    }
                });
            });
    }

    fn toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar")
            .exact_height(46.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.button("＋ MP3").clicked() {
                        self.open_files();
                    }

                    if ui.button("＋ Carpeta").clicked() {
                        self.open_folder();
                    }

                    ui.separator();

                    if ui
                        .add_enabled(
                            self.controller.selected().is_some(),
                            egui::Button::new("Editar"),
                        )
                        .clicked()
                    {
                        self.show_editor = true;
                    }

                    if ui
                        .add_enabled(
                            self.controller.selected().is_some() && self.batch.is_none(),
                            egui::Button::new("Normalizar"),
                        )
                        .clicked()
                    {
                        self.controller.normalize_selected(&self.normalization);
                    }

                    if ui
                        .add_enabled(
                            !self.controller.files().is_empty() && self.batch.is_none(),
                            egui::Button::new("Lote"),
                        )
                        .clicked()
                    {
                        self.confirm_normalize_all = true;
                    }

                    if ui
                        .add_enabled(
                            self.controller.last_output_dir().is_some(),
                            egui::Button::new("Abrir salida"),
                        )
                        .clicked()
                    {
                        self.controller.open_output_folder();
                    }

                    ui.separator();

                    ui.label("Buscar");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.filter)
                            .desired_width(240.0)
                            .hint_text("archivo, título, artista, álbum"),
                    );

                    if !self.filter.is_empty() && ui.small_button("×").clicked() {
                        self.filter.clear();
                    }
                });
            });
    }

    fn files_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("files")
            .resizable(true)
            .default_width(350.0)
            .min_width(260.0)
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
                        let selected_row = selected == Some(index);
                        let response = ui.selectable_label(
                            selected_row,
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

                        if response.double_clicked() {
                            self.show_editor = true;
                        }

                        ui.separator();
                    }
                });
            });
    }

    fn overview(&mut self, ui: &mut egui::Ui) {
        let Some(file) = self.controller.selected() else {
            ui.vertical_centered(|ui| {
                ui.add_space(110.0);
                ui.heading("Normalizador NQ4");
                ui.label("Añade MP3 o una carpeta completa para comenzar.");
                ui.add_space(8.0);
                ui.label(format!(
                    "Los resultados se guardan en una carpeta «{NORMALIZED_FOLDER_NAME}»."
                ));
            });
            return;
        };

        let name = file.display_name();
        let path = file.path.display().to_string();
        let title = file.tags.title.clone();
        let artist = file.tags.artist.clone();
        let album = file.tags.album.clone();
        let genre = file.tags.genre.clone();
        let year = file.tags.year.clone();
        let version = file.diagnostics.id3_version.clone();
        let tlen = file.diagnostics.raw_tlen.clone();
        let warnings = file.diagnostics.warnings();
        let dirty = file.dirty;

        ui.horizontal(|ui| {
            ui.heading(name);
            if dirty {
                ui.colored_label(
                    egui::Color32::from_rgb(235, 185, 80),
                    "● cambios sin guardar",
                );
            }
        });
        ui.monospace(path);
        ui.add_space(14.0);

        ui.horizontal_wrapped(|ui| {
            if ui.button("Editar tags…").clicked() {
                self.show_editor = true;
            }
            if ui.button("Normalizar este MP3").clicked() {
                self.controller.normalize_selected(&self.normalization);
            }
            if ui.button("Diagnóstico").clicked() {
                self.show_diagnostics = true;
            }
        });

        ui.add_space(14.0);

        egui::Grid::new("overview_grid")
            .num_columns(2)
            .spacing([30.0, 9.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Título");
                ui.label(if title.is_empty() { "—" } else { &title });
                ui.end_row();

                ui.label("Artista");
                ui.label(if artist.is_empty() { "—" } else { &artist });
                ui.end_row();

                ui.label("Álbum");
                ui.label(if album.is_empty() { "—" } else { &album });
                ui.end_row();

                ui.label("Género");
                ui.label(if genre.is_empty() { "—" } else { &genre });
                ui.end_row();

                ui.label("Año");
                ui.label(if year.is_empty() { "—" } else { &year });
                ui.end_row();

                ui.label("ID3");
                ui.label(version);
                ui.end_row();

                ui.label("TLEN");
                ui.label(tlen.as_deref().unwrap_or("No presente"));
                ui.end_row();
            });

        ui.add_space(14.0);

        if warnings.is_empty() {
            ui.colored_label(
                egui::Color32::from_rgb(120, 200, 140),
                "Sin alertas ID3 detectadas.",
            );
        } else {
            ui.heading("Alertas");
            for warning in warnings {
                ui.colored_label(
                    egui::Color32::from_rgb(235, 185, 80),
                    format!("⚠ {warning}"),
                );
            }
        }
    }

    fn editor_window(&mut self, ctx: &egui::Context) {
        if !self.show_editor {
            return;
        }

        let mut open = self.show_editor;
        let mut choose_cover = false;
        let mut remove_cover = false;
        let mut preview_cover = false;
        let mut save = false;
        let mut discard = false;

        egui::Window::new("Editar tags")
            .open(&mut open)
            .default_width(650.0)
            .min_width(520.0)
            .resizable(true)
            .show(ctx, |ui| {
                let Some(file) = self.controller.selected_mut() else {
                    ui.label("Selecciona un archivo.");
                    return;
                };

                ui.heading(file.display_name());
                ui.monospace(file.path.display().to_string());
                ui.separator();

                let mut changed = false;

                egui::Grid::new("editor_grid")
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

                if changed {
                    file.dirty = true;
                }

                ui.add_space(12.0);
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
                        choose_cover = ui.button("Cambiar…").clicked();
                        remove_cover =
                            file.tags.cover.is_some() && ui.button("Eliminar").clicked();
                        preview_cover =
                            file.tags.cover.is_some() && ui.button("Vista previa").clicked();
                    });
                });

                ui.add_space(14.0);
                ui.separator();
                ui.horizontal(|ui| {
                    save = ui
                        .add_enabled(file.dirty, egui::Button::new("Guardar cambios"))
                        .clicked();
                    discard = ui
                        .add_enabled(file.dirty, egui::Button::new("Descartar"))
                        .clicked();

                    if file.dirty {
                        ui.colored_label(
                            egui::Color32::from_rgb(235, 185, 80),
                            "Hay cambios sin guardar",
                        );
                    }
                });
            });

        self.show_editor = open;

        if choose_cover {
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

        if preview_cover {
            self.show_cover = true;
            self.cover_texture_key = None;
        }

        if save {
            self.controller.save_selected();
        }

        if discard {
            self.controller.reload_selected();
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

    fn preferences_window(&mut self, ctx: &egui::Context) {
        if !self.show_preferences {
            return;
        }

        let mut open = self.show_preferences;
        egui::Window::new("Preferencias de normalización")
            .open(&mut open)
            .default_width(500.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Perfil iPod seguro");
                ui.label("La normalización siempre crea una copia; el MP3 original queda intacto.");
                ui.add_space(8.0);

                ui.label(format!(
                    "Carpeta de salida: {NORMALIZED_FOLDER_NAME}"
                ));
                ui.separator();

                ui.checkbox(&mut self.normalization.strip_id3v1, "Eliminar ID3v1");
                ui.checkbox(&mut self.normalization.strip_apev2, "Eliminar APEv2");
                ui.checkbox(
                    &mut self.normalization.normalize_cover,
                    "Normalizar portada a JPEG",
                );

                ui.add_enabled_ui(self.normalization.normalize_cover, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Máximo");
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
                ui.small("Se escribe ID3v2.3 y se elimina TLEN.");
                ui.small("El audio MP3 no se recodifica.");
            });
        self.show_preferences = open;
    }

    fn diagnostics_window(&mut self, ctx: &egui::Context) {
        if !self.show_diagnostics {
            return;
        }

        let mut open = self.show_diagnostics;
        egui::Window::new("Diagnóstico ID3")
            .open(&mut open)
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
                    file.diagnostics.raw_tlen.as_deref().unwrap_or("no presente")
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
        self.show_diagnostics = open;
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
            self.cover_texture = image::load_from_memory(&cover.data).ok().map(|image| {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                ctx.load_texture("cover_preview", color_image, egui::TextureOptions::LINEAR)
            });
            self.cover_texture_key = Some(key);
        }

        let mut open = self.show_cover;
        egui::Window::new("Vista previa de portada")
            .open(&mut open)
            .resizable(true)
            .default_size([520.0, 560.0])
            .show(ctx, |ui| {
                if let Some(texture) = &self.cover_texture {
                    let available = ui.available_size();
                    let source = texture.size_vec2();
                    let scale = (available.x / source.x)
                        .min((available.y - 50.0).max(100.0) / source.y)
                        .min(1.0);
                    ui.add(egui::Image::new((texture.id(), source * scale)));
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
        self.show_cover = open;
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }

        let mut open = self.show_about;
        egui::Window::new("Acerca de")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Normalizador NQ4");
                ui.label("Editor y normalizador portable de etiquetas ID3.");
                ui.add_space(8.0);
                ui.label("Rust · MVC · SOLID");
                ui.label("Perfil de compatibilidad para iPod y reproductores antiguos.");
            });
        self.show_about = open;
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
                    "Se crearán {} MP3 normalizados.",
                    self.controller.files().len()
                ));
                ui.label(format!(
                    "Los originales no se tocarán. Salida: «{NORMALIZED_FOLDER_NAME}»."
                ));
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

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status")
            .exact_height(34.0)
            .show(ctx, |ui| {
                if let Some(batch) = &self.batch {
                    ui.horizontal_centered(|ui| {
                        ui.label(format!(
                            "Normalizando {}/{}",
                            batch.completed, batch.total
                        ));
                        ui.add(
                            egui::ProgressBar::new(batch.progress())
                                .desired_width(240.0)
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
                    ui.horizontal_centered(|ui| {
                        ui.label(self.controller.status());
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.label(format!(
                                    "{} archivo(s) · {} alerta(s)",
                                    self.controller.files().len(),
                                    self.controller.warning_count()
                                ));
                            },
                        );
                    });
                }
            });
    }
}

impl eframe::App for Nq4App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.configure_style(ctx);
        self.handle_drop(ctx);
        self.handle_shortcuts(ctx);
        self.process_batch_step(ctx);

        self.title_bar(ctx);
        self.menu_bar(ctx);
        self.toolbar(ctx);
        self.status_bar(ctx);
        self.files_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.overview(ui));
        });

        self.editor_window(ctx);
        self.confirm_normalize_all_window(ctx);
        self.preferences_window(ctx);
        self.diagnostics_window(ctx);
        self.cover_window(ctx);
        self.about_window(ctx);
    }
}
