use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    model::{AudioFile, CoverArt, NormalizationOptions},
    service::{
        AudioNormalizer, FileScanner, Id3TagRepository, IpodSafeNormalizer, Mp3FileScanner,
        TagRepository,
    },
};

pub struct MainController {
    files: Vec<AudioFile>,
    selected: Option<usize>,
    repository: Box<dyn TagRepository>,
    scanner: Box<dyn FileScanner>,
    normalizer: Box<dyn AudioNormalizer>,
    status: String,
    last_output_dir: Option<PathBuf>,
}

impl Default for MainController {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            selected: None,
            repository: Box::new(Id3TagRepository),
            scanner: Box::new(Mp3FileScanner),
            normalizer: Box::new(IpodSafeNormalizer),
            status: "Listo. Añade MP3, una carpeta o arrastra archivos a la ventana.".to_owned(),
            last_output_dir: None,
        }
    }
}

impl MainController {
    pub fn files(&self) -> &[AudioFile] {
        &self.files
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn selected(&self) -> Option<&AudioFile> {
        self.selected.and_then(|index| self.files.get(index))
    }

    pub fn selected_mut(&mut self) -> Option<&mut AudioFile> {
        self.selected.and_then(|index| self.files.get_mut(index))
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn dirty_count(&self) -> usize {
        self.files.iter().filter(|file| file.dirty).count()
    }

    pub fn warning_count(&self) -> usize {
        self.files
            .iter()
            .map(|file| file.diagnostics.warning_count())
            .sum()
    }

    pub fn last_output_dir(&self) -> Option<&Path> {
        self.last_output_dir.as_deref()
    }

    pub fn select(&mut self, index: usize) {
        if index < self.files.len() {
            self.selected = Some(index);
        }
    }

    pub fn add_paths(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        let mut added = 0_usize;
        let mut rejected = 0_usize;

        for path in paths {
            if path.is_dir() {
                let discovered = self.scanner.scan(&path);
                let (ok, bad) = self.add_files(discovered);
                added += ok;
                rejected += bad;
            } else if Self::is_mp3(&path) {
                let (ok, bad) = self.add_files([path]);
                added += ok;
                rejected += bad;
            }
        }

        if self.selected.is_none() && !self.files.is_empty() {
            self.selected = Some(0);
        }

        self.status = match (added, rejected) {
            (0, 0) => "No se agregaron MP3 nuevos.".to_owned(),
            (_, 0) => format!("Se agregaron {added} archivo(s)."),
            _ => format!("Se agregaron {added}; {rejected} no se pudieron leer."),
        };
    }

    pub fn add_folder(&mut self, root: &Path) {
        self.add_paths(self.scanner.scan(root));
    }

    pub fn save_selected(&mut self) {
        let Some(index) = self.selected else {
            self.status = "No hay archivo seleccionado.".to_owned();
            return;
        };

        match self.save_index(index) {
            Ok(()) => self.status = "Tags guardados como ID3v2.3.".to_owned(),
            Err(error) => self.status = format!("No se pudo guardar: {error}"),
        }
    }

    pub fn save_all(&mut self) {
        let indices: Vec<usize> = self
            .files
            .iter()
            .enumerate()
            .filter_map(|(index, file)| file.dirty.then_some(index))
            .collect();

        if indices.is_empty() {
            self.status = "No hay cambios pendientes.".to_owned();
            return;
        }

        let mut ok = 0;
        let mut failed = Vec::new();

        for index in indices {
            match self.save_index(index) {
                Ok(()) => ok += 1,
                Err(error) => failed.push(format!("{}: {error}", self.files[index].display_name())),
            }
        }

        self.status = if failed.is_empty() {
            format!("{ok} archivo(s) guardado(s).")
        } else {
            format!("{ok} guardado(s); {} con error. {}", failed.len(), failed[0])
        };
    }

    pub fn reload_selected(&mut self) {
        let Some(index) = self.selected else {
            self.status = "No hay archivo seleccionado.".to_owned();
            return;
        };

        let path = self.files[index].path.clone();
        match self.repository.load(&path) {
            Ok(reloaded) => {
                self.files[index] = reloaded;
                self.status = "Cambios locales descartados.".to_owned();
            }
            Err(error) => self.status = format!("No se pudo recargar: {error}"),
        }
    }

    pub fn remove_selected(&mut self) {
        let Some(index) = self.selected else {
            return;
        };

        self.files.remove(index);

        self.selected = if self.files.is_empty() {
            None
        } else if index >= self.files.len() {
            Some(self.files.len() - 1)
        } else {
            Some(index)
        };

        self.status = "Archivo quitado de la lista. El MP3 no fue eliminado.".to_owned();
    }

    pub fn clear(&mut self) {
        self.files.clear();
        self.selected = None;
        self.status = "Lista vaciada. No se eliminaron archivos del disco.".to_owned();
    }

    pub fn normalize_selected(&mut self, options: &NormalizationOptions) {
        let Some(index) = self.selected else {
            self.status = "No hay archivo seleccionado.".to_owned();
            return;
        };

        match self.normalize_index(index, options) {
            Ok(target) => {
                self.remember_output(&target);
                self.status = format!("MP3 normalizado: {}", target.display());
            }
            Err(error) => self.status = format!("Error al normalizar: {error}"),
        }
    }

    pub fn normalize_one(
        &mut self,
        index: usize,
        options: &NormalizationOptions,
    ) -> Result<PathBuf, String> {
        let target = self.normalize_index(index, options)?;
        self.remember_output(&target);
        Ok(target)
    }

    pub fn open_output_folder(&mut self) {
        let Some(path) = self.last_output_dir.clone() else {
            self.status = "Todavía no hay una carpeta de salida.".to_owned();
            return;
        };

        #[cfg(target_os = "windows")]
        {
            match Command::new("explorer").arg(&path).spawn() {
                Ok(_) => self.status = format!("Abriendo {}", path.display()),
                Err(error) => self.status = format!("No se pudo abrir la carpeta: {error}"),
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            self.status = format!("Carpeta de salida: {}", path.display());
        }
    }

    pub fn set_status(&mut self, value: impl Into<String>) {
        self.status = value.into();
    }

    pub fn set_cover(&mut self, path: &Path) {
        let Some(file) = self.selected_mut() else {
            self.status = "No hay archivo seleccionado.".to_owned();
            return;
        };

        match fs::read(path) {
            Ok(data) => {
                let mime_type = match path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    _ => {
                        self.status = "La portada debe ser PNG o JPEG.".to_owned();
                        return;
                    }
                };

                file.tags.cover = Some(CoverArt {
                    mime_type: mime_type.to_owned(),
                    description: "Cover".to_owned(),
                    data,
                });
                file.dirty = true;
                self.status = "Portada cargada. Hay cambios sin guardar.".to_owned();
            }
            Err(error) => self.status = format!("No se pudo leer la portada: {error}"),
        }
    }

    pub fn remove_cover(&mut self) {
        if let Some(file) = self.selected_mut() {
            file.tags.cover = None;
            file.dirty = true;
            self.status = "Portada eliminada. Hay cambios sin guardar.".to_owned();
        }
    }

    fn remember_output(&mut self, target: &Path) {
        self.last_output_dir = target.parent().map(Path::to_path_buf);
    }

    fn save_index(&mut self, index: usize) -> Result<(), String> {
        self.repository
            .save_preserving_unknown_frames(&self.files[index])
            .map_err(|error| error.to_string())?;

        let path = self.files[index].path.clone();
        self.files[index] = self
            .repository
            .load(&path)
            .map_err(|error| format!("Guardado, pero no se pudo recargar: {error}"))?;

        Ok(())
    }

    fn normalize_index(
        &mut self,
        index: usize,
        options: &NormalizationOptions,
    ) -> Result<PathBuf, String> {
        let file = self.files[index].clone();

        self.normalizer
            .normalize(&file, options, self.repository.as_ref())
            .map_err(|error| error.to_string())
    }

    fn add_files(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> (usize, usize) {
        let mut added = 0;
        let mut rejected = 0;

        for path in paths {
            if self.files.iter().any(|existing| {
                existing
                    .path
                    .to_string_lossy()
                    .eq_ignore_ascii_case(path.to_string_lossy().as_ref())
            }) {
                continue;
            }

            match self.repository.load(&path) {
                Ok(file) => {
                    self.files.push(file);
                    added += 1;
                }
                Err(_) => rejected += 1,
            }
        }

        (added, rejected)
    }

    fn is_mp3(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3"))
    }
}
