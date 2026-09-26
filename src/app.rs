use std::{collections::VecDeque, path::PathBuf};

use eframe::egui;
use rfd::FileDialog;

use crate::{
    controller::MainController,
    model::{CoverArt, NormalizationOptions, NORMALIZED_FOLDER_NAME},
};

const BG: egui::Color32 = egui::Color32::from_rgb(22, 26, 31);
const SIDEBAR: egui::Color32 = egui::Color32::from_rgb(27, 31, 36);
const CARD: egui::Color32 = egui::Color32::from_rgb(39, 44, 50);
const CARD_ALT: egui::Color32 = egui::Color32::from_rgb(34, 39, 45);
const ACCENT: egui::Color32 = egui::Color32::from_rgb(29, 203, 137);
const MUTED: egui::Color32 = egui::Color32::from_rgb(145, 153, 163);
const TEXT: egui::Color32 = egui::Color32::from_rgb(238, 241, 244);
const WARNING: egui::Color32 = egui::Color32::from_rgb(242, 183, 64);
const DANGER: egui::Color32 = egui::Color32::from_rgb(232, 89, 89);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Workspace {
    Overview,
    Library,
    Normalize,
}

#[derive(Clone, Copy)]
enum WindowButtonKind {
    Minimize,
    Maximize,
    Close,
}

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
    workspace: Workspace,
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
    maximized: bool,
}

impl Nq4App {
    pub fn new(controller: MainController) -> Self {
        Self {
            controller,
            normalization: NormalizationOptions::default(),
            workspace: Workspace::Overview,
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
            maximized: false,
        }
    }

    fn configure_style(&mut self, ctx: &egui::Context) {
        if self.style_configured {
            return;
        }

        let mut style = (*ctx.style()).clone();
        style.spacing.button_padding = egui::vec2(12.0, 7.0);
        style.spacing.item_spacing = egui::vec2(10.0, 9.0);
        style.spacing.interact_size.y = 32.0;
        style.spacing.window_margin = egui::Margin::same(16);
        ctx.set_style(style);

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG;
        visuals.window_fill = CARD_ALT;
        visuals.extreme_bg_color = egui::Color32::from_rgb(15, 18, 22);
        visuals.faint_bg_color = CARD;
        visuals.widgets.noninteractive.bg_fill = CARD;
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(48, 53, 60);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(57, 63, 70);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(64, 71, 78);
        visuals.selection.bg_fill = ACCENT;
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
        ctx.set_visuals(visuals);

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

    fn window_button(
        ui: &mut egui::Ui,
        kind: WindowButtonKind,
        maximized: bool,
    ) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(46.0, 38.0), egui::Sense::click());

        if response.hovered() {
            let fill = match kind {
                WindowButtonKind::Close => egui::Color32::from_rgb(196, 43, 50),
                _ => egui::Color32::from_rgb(48, 54, 61),
            };
            ui.painter().rect_filled(rect, 0.0, fill);
        }

        let icon_color = if response.hovered() && matches!(kind, WindowButtonKind::Close) {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_rgb(205, 211, 218)
        };
        let stroke = egui::Stroke::new(1.25_f32, icon_color);
        let center = rect.center();

        match kind {
            WindowButtonKind::Minimize => {
                ui.painter().line_segment(
                    [
                        center + egui::vec2(-5.5, 3.0),
                        center + egui::vec2(5.5, 3.0),
                    ],
                    stroke,
                );
            }
            WindowButtonKind::Maximize => {
                if maximized {
                    let back = egui::Rect::from_min_size(
                        center + egui::vec2(-2.0, -5.0),
                        egui::vec2(9.0, 8.0),
                    );
                    let front = egui::Rect::from_min_size(
                        center + egui::vec2(-6.0, -1.0),
                        egui::vec2(9.0, 8.0),
                    );

                    ui.painter().rect_stroke(
                        back,
                        0.0,
                        stroke,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().rect_filled(
                        front.expand(1.0),
                        0.0,
                        if response.hovered() {
                            egui::Color32::from_rgb(48, 54, 61)
                        } else {
                            egui::Color32::from_rgb(20, 23, 27)
                        },
                    );
                    ui.painter().rect_stroke(
                        front,
                        0.0,
                        stroke,
                        egui::StrokeKind::Inside,
                    );
                } else {
                    let box_rect =
                        egui::Rect::from_center_size(center, egui::vec2(10.0, 9.0));
                    ui.painter().rect_stroke(
                        box_rect,
                        0.0,
                        stroke,
                        egui::StrokeKind::Inside,
                    );
                }
            }
            WindowButtonKind::Close => {
                ui.painter().line_segment(
                    [
                        center + egui::vec2(-5.0, -5.0),
                        center + egui::vec2(5.0, 5.0),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        center + egui::vec2(5.0, -5.0),
                        center + egui::vec2(-5.0, 5.0),
                    ],
                    stroke,
                );
            }
        }

        response
    }

    fn title_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("custom_title_bar")
            .exact_height(38.0)
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(20, 23, 27)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("NQ4").strong().color(ACCENT));
                    ui.label(
                        egui::RichText::new("Normalizador de MP3")
                            .color(egui::Color32::from_rgb(185, 192, 200)),
                    );

                    let controls_width = 138.0;
                    let drag_width = (ui.available_width() - controls_width).max(80.0);
                    let (_, response) = ui.allocate_exact_size(
                        egui::vec2(drag_width, 38.0),
                        egui::Sense::click_and_drag(),
                    );

                    if response.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }

                    if response.double_clicked() {
                        self.maximized = !self.maximized;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(self.maximized));
                    }

                    if Self::window_button(ui, WindowButtonKind::Minimize, self.maximized)
                        .on_hover_text("Minimizar")
                        .clicked()
                    {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }

                    if Self::window_button(ui, WindowButtonKind::Maximize, self.maximized)
                        .on_hover_text(if self.maximized { "Restaurar" } else { "Maximizar" })
                        .clicked()
                    {
                        self.maximized = !self.maximized;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(self.maximized));
                    }

                    if Self::window_button(ui, WindowButtonKind::Close, self.maximized)
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
            .exact_height(32.0)
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(24, 28, 33)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);

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
                                egui::Button::new("Guardar seleccionado   Ctrl+S"),
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
                                egui::Button::new("Normalizar seleccionado"),
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

                        if ui
                            .add_enabled(
                                self.controller.last_output_dir().is_some(),
                                egui::Button::new("Abrir MP3 normalizados"),
                            )
                            .clicked()
                        {
                            self.controller.open_output_folder();
                            ui.close();
                        }

                        ui.separator();

                        if ui.button("Preferencias…").clicked() {
                            self.show_preferences = true;
                            ui.close();
                        }
                    });

                    ui.menu_button("Ayuda", |ui| {
                        if ui.button("Acerca de NQ4").clicked() {
                            self.show_about = true;
                            ui.close();
                        }
                    });
                });
            });
    }

    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .exact_width(205.0)
            .frame(
                egui::Frame::new()
                    .fill(SIDEBAR)
                    .inner_margin(egui::Margin::symmetric(14, 16)),
            )
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("NQ4")
                        .size(22.0)
                        .strong()
                        .color(TEXT),
                );
                ui.label(
                    egui::RichText::new("ID3 Toolkit")
                        .small()
                        .color(MUTED),
                );
                ui.add_space(18.0);

                Self::nav_button(ui, &mut self.workspace, Workspace::Overview, "▦  Resumen");
                Self::nav_button(
                    ui,
                    &mut self.workspace,
                    Workspace::Library,
                    &format!("♫  Biblioteca   {}", self.controller.files().len()),
                );
                Self::nav_button(
                    ui,
                    &mut self.workspace,
                    Workspace::Normalize,
                    &format!("✓  Normalizar   {}", self.controller.warning_count()),
                );

                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("ACCIONES")
                        .small()
                        .strong()
                        .color(MUTED),
                );
                ui.add_space(4.0);

                if ui
                    .add_sized(
                        [ui.available_width(), 34.0],
                        egui::Button::new("＋  Añadir MP3").frame(false),
                    )
                    .clicked()
                {
                    self.open_files();
                }

                if ui
                    .add_sized(
                        [ui.available_width(), 34.0],
                        egui::Button::new("▣  Añadir carpeta").frame(false),
                    )
                    .clicked()
                {
                    self.open_folder();
                }

                if ui
                    .add_enabled(
                        self.controller.selected().is_some(),
                        egui::Button::new("✎  Editar tags").frame(false),
                    )
                    .clicked()
                {
                    self.show_editor = true;
                }

                if ui
                    .add_enabled(
                        self.controller.selected().is_some(),
                        egui::Button::new("⚙  Diagnóstico").frame(false),
                    )
                    .clicked()
                {
                    self.show_diagnostics = true;
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            egui::Button::new("⚙  Preferencias").frame(false),
                        )
                        .clicked()
                    {
                        self.show_preferences = true;
                    }

                    ui.add_space(8.0);

                    let dirty = self.controller.dirty_count();
                    if dirty > 0 {
                        ui.label(
                            egui::RichText::new(format!("{dirty} archivo(s) sin guardar"))
                                .small()
                                .color(WARNING),
                        );
                    }
                });
            });
    }

    fn nav_button(
        ui: &mut egui::Ui,
        current: &mut Workspace,
        target: Workspace,
        text: &str,
    ) {
        let selected = *current == target;
        let button = egui::Button::new(
            egui::RichText::new(text)
                .color(if selected { egui::Color32::WHITE } else { egui::Color32::from_rgb(195, 201, 208) }),
        )
        .fill(if selected { ACCENT } else { egui::Color32::TRANSPARENT })
        .frame(selected);

        if ui
            .add_sized([ui.available_width(), 38.0], button)
            .clicked()
        {
            *current = target;
        }
    }

    fn page_header(&mut self, ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).size(25.0).strong().color(TEXT));
                ui.label(egui::RichText::new(subtitle).color(MUTED));
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !self.filter.is_empty() && ui.small_button("×").clicked() {
                    self.filter.clear();
                }

                ui.add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .desired_width(260.0)
                        .hint_text("Buscar en la biblioteca…"),
                );
                ui.label("⌕");
            });
        });
        ui.add_space(14.0);
    }

    fn card<R>(
        ui: &mut egui::Ui,
        title: &str,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        egui::Frame::new()
            .fill(CARD)
            .corner_radius(10.0)
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(title).strong().color(TEXT));
                ui.add_space(10.0);
                add_contents(ui)
            })
    }

    fn cover_texture(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        cover: Option<&CoverArt>,
    ) -> Option<egui::TextureHandle> {
        let Some(cover) = cover else {
            self.cover_texture = None;
            self.cover_texture_key = None;
            return None;
        };

        if self.cover_texture_key.as_deref() != Some(key) {
            self.cover_texture = image::load_from_memory(&cover.data).ok().map(|image| {
                let rgba = image.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                ctx.load_texture(
                    format!("cover:{key}"),
                    color_image,
                    egui::TextureOptions::LINEAR,
                )
            });
            self.cover_texture_key = Some(key.to_owned());
        }

        self.cover_texture.clone()
    }

    fn cover_placeholder(ui: &mut egui::Ui, size: egui::Vec2) {
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        ui.painter().rect(
            rect,
            10.0,
            egui::Color32::from_rgb(29, 34, 39),
            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 67, 74)),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "♫",
            egui::FontId::proportional(50.0),
            MUTED,
        );
    }

    fn overview_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.page_header(
            ui,
            "Resumen",
            "Vista general de la biblioteca y del perfil de compatibilidad.",
        );

        let selected = self.controller.selected_index();
        let files_count = self.controller.files().len();
        let warnings_count = self.controller.warning_count();

        ui.columns(3, |columns| {
            Self::card(&mut columns[0], "Biblioteca", |ui| {
                ui.label(
                    egui::RichText::new(format!("{files_count}"))
                        .size(30.0)
                        .strong()
                        .color(ACCENT),
                );
                ui.label(egui::RichText::new("MP3 cargados").color(MUTED));
                ui.add_space(12.0);
                if ui.button("Abrir biblioteca").clicked() {
                    self.workspace = Workspace::Library;
                }
            });

            Self::card(&mut columns[1], "Compatibilidad", |ui| {
                ui.label(
                    egui::RichText::new(format!("{warnings_count}"))
                        .size(30.0)
                        .strong()
                        .color(if warnings_count == 0 { ACCENT } else { WARNING }),
                );
                ui.label(egui::RichText::new("alertas ID3 detectadas").color(MUTED));
                ui.add_space(12.0);
                if ui.button("Revisar normalización").clicked() {
                    self.workspace = Workspace::Normalize;
                }
            });

            Self::card(&mut columns[2], "Salida", |ui| {
                ui.label(
                    egui::RichText::new(NORMALIZED_FOLDER_NAME)
                        .size(18.0)
                        .strong()
                        .color(TEXT),
                );
                ui.label(egui::RichText::new("los originales se conservan").color(MUTED));
                ui.add_space(12.0);
                if ui
                    .add_enabled(
                        self.controller.last_output_dir().is_some(),
                        egui::Button::new("Abrir carpeta"),
                    )
                    .clicked()
                {
                    self.controller.open_output_folder();
                }
            });
        });

        ui.add_space(14.0);

        ui.columns(3, |columns| {
            Self::card(&mut columns[0], "Tema seleccionado", |ui| {
                let Some(index) = selected else {
                    ui.label(egui::RichText::new("Selecciona un MP3 en Biblioteca.").color(MUTED));
                    return;
                };

                let (name, title, artist, album, cover, path) = {
                    let file = &self.controller.files()[index];
                    (
                        file.display_name(),
                        file.tags.title.clone(),
                        file.tags.artist.clone(),
                        file.tags.album.clone(),
                        file.tags.cover.clone(),
                        file.path.display().to_string(),
                    )
                };

                let key = format!("{path}:{}", cover.as_ref().map(|c| c.data.len()).unwrap_or(0));
                let texture = self.cover_texture(ctx, &key, cover.as_ref());

                ui.horizontal_top(|ui| {
                    let cover_size = egui::vec2(110.0, 110.0);
                    if let Some(texture) = &texture {
                        if ui
                            .add(egui::Image::new((texture.id(), cover_size)).corner_radius(8.0))
                            .clicked()
                        {
                            self.show_cover = true;
                        }
                    } else {
                        Self::cover_placeholder(ui, cover_size);
                    }

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(if title.trim().is_empty() { &name } else { &title })
                                .strong()
                                .color(TEXT),
                        );
                        if !artist.trim().is_empty() {
                            ui.label(egui::RichText::new(&artist).color(MUTED));
                        }
                        if !album.trim().is_empty() {
                            ui.label(egui::RichText::new(&album).small().color(MUTED));
                        }
                        ui.add_space(8.0);
                        if ui.button("Editar tags").clicked() {
                            self.show_editor = true;
                        }
                    });
                });
            });

            Self::card(&mut columns[1], "Normalización rápida", |ui| {
                ui.label("ID3v2.3");
                ui.label(egui::RichText::new("TLEN eliminado").color(MUTED));
                ui.label(egui::RichText::new("Carátula JPEG compatible").color(MUTED));
                ui.add_space(12.0);

                if ui
                    .add_enabled(
                        !self.controller.files().is_empty() && self.batch.is_none(),
                        egui::Button::new("Normalizar todos"),
                    )
                    .clicked()
                {
                    self.confirm_normalize_all = true;
                }
            });

            Self::card(&mut columns[2], "Estado", |ui| {
                ui.label(
                    egui::RichText::new(self.controller.status())
                        .color(egui::Color32::from_rgb(203, 208, 214)),
                );

                if let Some(batch) = &self.batch {
                    ui.add_space(12.0);
                    ui.add(
                        egui::ProgressBar::new(batch.progress())
                            .show_percentage()
                            .desired_width(ui.available_width()),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{} de {}",
                            batch.completed, batch.total
                        ))
                        .small()
                        .color(MUTED),
                    );
                }
            });
        });

        ui.add_space(14.0);

        Self::card(ui, "Biblioteca reciente", |ui| {
            let rows: Vec<(usize, String, String, String, usize)> = self
                .controller
                .files()
                .iter()
                .enumerate()
                .take(8)
                .map(|(index, file)| {
                    (
                        index,
                        if file.tags.title.trim().is_empty() {
                            file.display_name()
                        } else {
                            file.tags.title.clone()
                        },
                        file.tags.artist.clone(),
                        file.tags.album.clone(),
                        file.diagnostics.warning_count(),
                    )
                })
                .collect();

            if rows.is_empty() {
                ui.label(egui::RichText::new("Aún no hay MP3 cargados.").color(MUTED));
                return;
            }

            egui::Grid::new("recent_tracks")
                .num_columns(4)
                .spacing([18.0, 9.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Título").strong());
                    ui.label(egui::RichText::new("Artista").strong());
                    ui.label(egui::RichText::new("Álbum").strong());
                    ui.label(egui::RichText::new("Estado").strong());
                    ui.end_row();

                    for (index, title, artist, album, warnings) in rows {
                        if ui.selectable_label(selected == Some(index), title).clicked() {
                            self.controller.select(index);
                            self.cover_texture_key = None;
                        }
                        ui.label(if artist.is_empty() { "—" } else { &artist });
                        ui.label(if album.is_empty() { "—" } else { &album });
                        ui.colored_label(
                            if warnings == 0 { ACCENT } else { WARNING },
                            if warnings == 0 {
                                "Compatible".to_owned()
                            } else {
                                format!("{warnings} alerta(s)")
                            },
                        );
                        ui.end_row();
                    }
                });
        });
    }

    fn library_page(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.page_header(
            ui,
            "Biblioteca",
            "Revisa tus MP3, carátulas y metadatos sin perder espacio en la interfaz.",
        );

        if self.controller.files().is_empty() {
            Self::card(ui, "Sin archivos", |ui| {
                ui.label(egui::RichText::new("Añade MP3 o una carpeta completa.").color(MUTED));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("Añadir MP3").clicked() {
                        self.open_files();
                    }
                    if ui.button("Añadir carpeta").clicked() {
                        self.open_folder();
                    }
                });
            });
            return;
        }

        if let Some(index) = self.controller.selected_index() {
            let (title, artist, album, path, cover) = {
                let file = &self.controller.files()[index];
                (
                    if file.tags.title.trim().is_empty() {
                        file.display_name()
                    } else {
                        file.tags.title.clone()
                    },
                    file.tags.artist.clone(),
                    file.tags.album.clone(),
                    file.path.display().to_string(),
                    file.tags.cover.clone(),
                )
            };

            let key = format!("{path}:{}", cover.as_ref().map(|c| c.data.len()).unwrap_or(0));
            let texture = self.cover_texture(ctx, &key, cover.as_ref());

            Self::card(ui, "Selección", |ui| {
                ui.horizontal_top(|ui| {
                    let size = egui::vec2(92.0, 92.0);
                    if let Some(texture) = &texture {
                        if ui
                            .add(egui::Image::new((texture.id(), size)).corner_radius(8.0))
                            .clicked()
                        {
                            self.show_cover = true;
                        }
                    } else {
                        Self::cover_placeholder(ui, size);
                    }

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(title).size(19.0).strong().color(TEXT));
                        ui.label(egui::RichText::new(if artist.is_empty() { "Artista desconocido" } else { &artist }).color(MUTED));
                        if !album.is_empty() {
                            ui.label(egui::RichText::new(album).small().color(MUTED));
                        }
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("Editar tags").clicked() {
                                self.show_editor = true;
                            }
                            if ui.button("Diagnóstico").clicked() {
                                self.show_diagnostics = true;
                            }
                            if ui
                                .add_enabled(self.batch.is_none(), egui::Button::new("Normalizar"))
                                .clicked()
                            {
                                self.controller.normalize_selected(&self.normalization);
                            }
                        });
                    });
                });
            });

            ui.add_space(14.0);
        }

        Self::card(ui, "Todos los MP3", |ui| {
            let filter = self.filter.trim().to_lowercase();
            let selected = self.controller.selected_index();

            let rows: Vec<(usize, String, String, String, String, usize)> = self
                .controller
                .files()
                .iter()
                .enumerate()
                .filter_map(|(index, file)| {
                    let title = if file.tags.title.trim().is_empty() {
                        file.display_name()
                    } else {
                        file.tags.title.clone()
                    };

                    let haystack = format!(
                        "{} {} {} {}",
                        title,
                        file.tags.artist,
                        file.tags.album,
                        file.display_name()
                    )
                    .to_lowercase();

                    if !filter.is_empty() && !haystack.contains(&filter) {
                        return None;
                    }

                    Some((
                        index,
                        title,
                        file.tags.artist.clone(),
                        file.tags.album.clone(),
                        file.diagnostics.id3_version.clone(),
                        file.diagnostics.warning_count(),
                    ))
                })
                .collect();

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .max_height(430.0)
                .show(ui, |ui| {
                    egui::Grid::new("library_table")
                        .num_columns(5)
                        .spacing([22.0, 9.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Título").strong());
                            ui.label(egui::RichText::new("Artista").strong());
                            ui.label(egui::RichText::new("Álbum").strong());
                            ui.label(egui::RichText::new("ID3").strong());
                            ui.label(egui::RichText::new("Estado").strong());
                            ui.end_row();

                            for (index, title, artist, album, version, warnings) in rows {
                                if ui.selectable_label(selected == Some(index), title).clicked() {
                                    self.controller.select(index);
                                    self.cover_texture_key = None;
                                }
                                ui.label(if artist.is_empty() { "—" } else { &artist });
                                ui.label(if album.is_empty() { "—" } else { &album });
                                ui.label(version);
                                ui.colored_label(
                                    if warnings == 0 { ACCENT } else { WARNING },
                                    if warnings == 0 {
                                        "OK".to_owned()
                                    } else {
                                        format!("⚠ {warnings}")
                                    },
                                );
                                ui.end_row();
                            }
                        });
                });
        });
    }

    fn normalize_page(&mut self, ui: &mut egui::Ui) {
        self.page_header(
            ui,
            "Normalización",
            "Perfil conservador para iPod y reproductores antiguos.",
        );

        ui.columns(3, |columns| {
            Self::card(&mut columns[0], "ID3", |ui| {
                ui.label(egui::RichText::new("ID3v2.3").size(22.0).strong().color(ACCENT));
                ui.label(egui::RichText::new("Se elimina TLEN").color(MUTED));
                ui.label(egui::RichText::new("No se recodifica el audio").color(MUTED));
            });

            Self::card(&mut columns[1], "Carátula", |ui| {
                ui.checkbox(
                    &mut self.normalization.normalize_cover,
                    "Normalizar a JPEG",
                );
                ui.horizontal(|ui| {
                    ui.label("Máximo");
                    ui.add(
                        egui::DragValue::new(&mut self.normalization.max_cover_size)
                            .range(128..=1200)
                            .suffix(" px"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Calidad");
                    ui.add(
                        egui::DragValue::new(&mut self.normalization.jpeg_quality)
                            .range(50..=100),
                    );
                });
            });

            Self::card(&mut columns[2], "Tags residuales", |ui| {
                ui.checkbox(&mut self.normalization.strip_id3v1, "Eliminar ID3v1");
                ui.checkbox(&mut self.normalization.strip_apev2, "Eliminar APEv2");
                ui.label(
                    egui::RichText::new(format!("Salida: {NORMALIZED_FOLDER_NAME}"))
                        .small()
                        .color(MUTED),
                );
            });
        });

        ui.add_space(14.0);

        Self::card(ui, "Procesar biblioteca", |ui| {
            let total = self.controller.files().len();
            let warnings = self.controller.warning_count();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{total} archivos"))
                            .size(24.0)
                            .strong()
                            .color(TEXT),
                    );
                    ui.label(
                        egui::RichText::new(format!("{warnings} alertas detectadas"))
                            .color(if warnings == 0 { ACCENT } else { WARNING }),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_enabled(
                            total > 0 && self.batch.is_none(),
                            egui::Button::new("Normalizar todos"),
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
                });
            });

            if let Some(batch) = &self.batch {
                ui.add_space(14.0);
                ui.add(
                    egui::ProgressBar::new(batch.progress())
                        .show_percentage()
                        .desired_width(ui.available_width()),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "{} de {} completados",
                        batch.completed, batch.total
                    ))
                    .small()
                    .color(MUTED),
                );
            }
        });

        ui.add_space(14.0);

        Self::card(ui, "Qué cambia", |ui| {
            ui.label("• Reescribe los metadatos como ID3v2.3.");
            ui.label("• Elimina TLEN para evitar duraciones incorrectas.");
            ui.label("• Puede retirar ID3v1 y APEv2.");
            ui.label("• Puede convertir la portada a JPEG compatible.");
            ui.label("• Crea copias en «MP3 normalizados» y mantiene los originales.");
        });
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
            .default_width(670.0)
            .min_width(560.0)
            .resizable(true)
            .show(ctx, |ui| {
                let Some(file) = self.controller.selected_mut() else {
                    ui.label("Selecciona un archivo.");
                    return;
                };

                ui.label(egui::RichText::new(file.display_name()).size(20.0).strong());
                ui.label(
                    egui::RichText::new(file.path.display().to_string())
                        .small()
                        .color(MUTED),
                );
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
                                    "Carátula: {} · {:.1} KiB",
                                    cover.mime_type,
                                    cover.data.len() as f32 / 1024.0
                                )
                            })
                            .unwrap_or_else(|| "Sin carátula".to_owned());

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
                        ui.colored_label(WARNING, "Hay cambios sin guardar");
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

    fn diagnostics_window(&mut self, ctx: &egui::Context) {
        if !self.show_diagnostics {
            return;
        }

        let mut open = self.show_diagnostics;

        egui::Window::new("Diagnóstico ID3")
            .open(&mut open)
            .default_width(540.0)
            .show(ctx, |ui| {
                let Some(file) = self.controller.selected() else {
                    ui.label("Selecciona un archivo.");
                    return;
                };

                ui.label(egui::RichText::new(file.display_name()).size(19.0).strong());
                ui.separator();

                egui::Grid::new("diag_grid")
                    .num_columns(2)
                    .spacing([22.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Versión ID3v2");
                        ui.label(&file.diagnostics.id3_version);
                        ui.end_row();

                        ui.label("TLEN");
                        ui.label(file.diagnostics.raw_tlen.as_deref().unwrap_or("No presente"));
                        ui.end_row();

                        ui.label("ID3v1");
                        ui.label(if file.diagnostics.has_id3v1 { "Presente" } else { "No" });
                        ui.end_row();

                        ui.label("APEv2");
                        ui.label(if file.diagnostics.has_apev2 { "Presente" } else { "No" });
                        ui.end_row();

                        ui.label("Carátula");
                        ui.label(
                            file.diagnostics
                                .cover_description
                                .as_deref()
                                .unwrap_or("No presente"),
                        );
                        ui.end_row();
                    });

                ui.add_space(12.0);
                let warnings = file.diagnostics.warnings();
                if warnings.is_empty() {
                    ui.colored_label(ACCENT, "✓ Sin alertas de compatibilidad");
                } else {
                    for warning in warnings {
                        ui.colored_label(WARNING, format!("⚠ {warning}"));
                    }
                }
            });

        self.show_diagnostics = open;
    }

    fn preferences_window(&mut self, ctx: &egui::Context) {
        if !self.show_preferences {
            return;
        }

        let mut open = self.show_preferences;

        egui::Window::new("Preferencias")
            .open(&mut open)
            .default_width(500.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Perfil iPod seguro").size(20.0).strong());
                ui.label(
                    egui::RichText::new(format!(
                        "Los resultados se guardan en «{NORMALIZED_FOLDER_NAME}»."
                    ))
                    .color(MUTED),
                );
                ui.separator();

                ui.checkbox(&mut self.normalization.strip_id3v1, "Eliminar ID3v1");
                ui.checkbox(&mut self.normalization.strip_apev2, "Eliminar APEv2");
                ui.checkbox(
                    &mut self.normalization.normalize_cover,
                    "Normalizar carátula a JPEG",
                );

                ui.add_enabled_ui(self.normalization.normalize_cover, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Tamaño máximo");
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
            });

        self.show_preferences = open;
    }

    fn cover_window(&mut self, ctx: &egui::Context) {
        if !self.show_cover {
            return;
        }

        let Some(index) = self.controller.selected_index() else {
            self.show_cover = false;
            return;
        };

        let (path, cover) = {
            let file = &self.controller.files()[index];
            (file.path.display().to_string(), file.tags.cover.clone())
        };

        let Some(cover) = cover else {
            self.show_cover = false;
            return;
        };

        let key = format!("{path}:{}", cover.data.len());
        let texture = self.cover_texture(ctx, &key, Some(&cover));

        let mut open = self.show_cover;

        egui::Window::new("Carátula")
            .open(&mut open)
            .resizable(true)
            .default_size([560.0, 600.0])
            .show(ctx, |ui| {
                if let Some(texture) = &texture {
                    let available = ui.available_size();
                    let source = texture.size_vec2();
                    let scale = (available.x / source.x)
                        .min((available.y - 55.0).max(100.0) / source.y)
                        .min(1.0);
                    ui.add(egui::Image::new((texture.id(), source * scale)));
                } else {
                    ui.colored_label(DANGER, "No se pudo decodificar la carátula.");
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

    fn confirm_normalize_all_window(&mut self, ctx: &egui::Context) {
        if !self.confirm_normalize_all {
            return;
        }

        egui::Window::new("Normalizar biblioteca")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(format!(
                    "Se crearán {} MP3 normalizados.",
                    self.controller.files().len()
                ));
                ui.label(format!(
                    "Los originales se conservarán. Salida: «{NORMALIZED_FOLDER_NAME}»."
                ));
                ui.add_space(12.0);

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
            .exact_height(32.0)
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(20, 23, 27)))
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if let Some(batch) = &self.batch {
                        ui.label(format!(
                            "Normalizando {}/{}",
                            batch.completed, batch.total
                        ));
                        ui.add(
                            egui::ProgressBar::new(batch.progress())
                                .desired_width(220.0)
                                .show_percentage(),
                        );
                    } else {
                        ui.label(egui::RichText::new(self.controller.status()).small().color(MUTED));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} MP3 · {} alertas",
                                self.controller.files().len(),
                                self.controller.warning_count()
                            ))
                            .small()
                            .color(MUTED),
                        );
                    });
                });
            });
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }

        let mut open = self.show_about;

        egui::Window::new("Acerca de NQ4")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Normalizador NQ4").size(21.0).strong());
                ui.label("Editor y normalizador portable de etiquetas ID3.");
                ui.add_space(8.0);
                ui.label("Rust · MVC · SOLID");
                ui.label("Perfil de compatibilidad para iPod y reproductores antiguos.");
            });

        self.show_about = open;
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
        self.status_bar(ctx);
        self.sidebar(ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(egui::Margin::symmetric(20, 18)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.workspace {
                        Workspace::Overview => self.overview_page(ctx, ui),
                        Workspace::Library => self.library_page(ctx, ui),
                        Workspace::Normalize => self.normalize_page(ui),
                    });
            });

        self.editor_window(ctx);
        self.diagnostics_window(ctx);
        self.preferences_window(ctx);
        self.cover_window(ctx);
        self.confirm_normalize_all_window(ctx);
        self.about_window(ctx);
    }
}
