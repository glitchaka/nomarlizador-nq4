use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    model::{AudioFile, CoverArt, NormalizationOptions},
    service::{
        AudioNormalizer, BackupService, FileBackupService, FileScanner, Id3TagRepository,
        IpodSafeNormalizer, Mp3FileScanner, TagRepository,
    },
};

pub struct MainController {
    files: Vec<AudioFile>,
    selected: Option<usize>,
    repository: Box<dyn TagRepository>,
    scanner: Box<dyn FileScanner>,
    normalizer: Box<dyn AudioNormalizer>,
    backup: Box<dyn BackupService>,
    status: String,
}

impl Default for MainController {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            selected: None,
            repository: Box::new(Id3TagRepository),
            scanner: Box::new(Mp3FileScanner),
            normalizer: Box::new(IpodSafeNormalizer),
            backup: Box::new(FileBackupService),
            status: "Arrastra MP3 o usa «Añadir MP3» / «Añadir carpeta».".to_owned(),
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

    pub fn select(&mut self, index: usize) {
        if index < self.files.len() {
            self.selected = Some(index);
        }
    }

    pub fn add_paths(&mut self, paths: impl IntoIterator<Item = PathBuf>) {
        let mut added = 0_usize;

        for path in paths {
            if path.is_dir() {
                let discovered = self.scanner.scan(&path);
                added += self.add_files(discovered);
            } else if Self::is_mp3(&path) {
                added += self.add_files([path]);
            }
        }

        if self.selected.is_none() && !self.files.is_empty() {
            self.selected = Some(0);
        }

        self.status = if added == 0 {
            "No se agregaron MP3 nuevos.".to_owned()
        } else {
            format!("Se agregaron {added} archivo(s).")
        };
    }

    pub fn add_folder(&mut self, root: &Path) {
        let files = self.scanner.scan(root);
        self.add_paths(files);
    }

    pub fn save_selected(&mut self) {
        let Some(index) = self.selected else {
            self.status = "No hay archivo seleccionado.".to_owned();
            return;
        };

        let result = self.repository.save_preserving_unknown_frames(&self.files[index]);
        match result {
            Ok(()) => match self.repository.load(&self.files[index].path) {
                Ok(reloaded) => {
                    self.files[index] = reloaded;
                    self.status = "Tags guardados como ID3v2.3.".to_owned();
                }
                Err(error) => self.status = format!("Guardado, pero no se pudo recargar: {error}"),
            },
            Err(error) => self.status = format!("No se pudo guardar: {error}"),
        }
    }

    pub fn normalize_selected(&mut self, options: &NormalizationOptions) {
        let Some(index) = self.selected else {
            self.status = "No hay archivo seleccionado.".to_owned();
            return;
        };

        match self.normalize_index(index, options) {
            Ok(()) => self.status = "Archivo normalizado sin recodificar el audio.".to_owned(),
            Err(error) => self.status = format!("Error al normalizar: {error}"),
        }
    }

    pub fn normalize_all(&mut self, options: &NormalizationOptions) {
        if self.files.is_empty() {
            self.status = "No hay archivos cargados.".to_owned();
            return;
        }

        let mut ok = 0_usize;
        let mut failed = Vec::new();

        for index in 0..self.files.len() {
            match self.normalize_index(index, options) {
                Ok(()) => ok += 1,
                Err(error) => failed.push(format!("{}: {error}", self.files[index].display_name())),
            }
        }

        self.status = if failed.is_empty() {
            format!("{ok} archivo(s) normalizado(s).")
        } else {
            format!(
                "{ok} normalizado(s); {} con error. Primero: {}",
                failed.len(),
                failed[0]
            )
        };
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
                self.status = "Portada cargada. Pulsa «Guardar tags» para escribirla.".to_owned();
            }
            Err(error) => self.status = format!("No se pudo leer la portada: {error}"),
        }
    }

    pub fn remove_cover(&mut self) {
        if let Some(file) = self.selected_mut() {
            file.tags.cover = None;
            self.status = "Portada eliminada en memoria. Pulsa «Guardar tags».".to_owned();
        }
    }

    fn normalize_index(
        &mut self,
        index: usize,
        options: &NormalizationOptions,
    ) -> Result<(), String> {
        let file = self.files[index].clone();

        self.normalizer
            .normalize(
                &file,
                options,
                self.repository.as_ref(),
                self.backup.as_ref(),
            )
            .map_err(|error| error.to_string())?;

        self.files[index] = self
            .repository
            .load(&file.path)
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    fn add_files(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> usize {
        let mut added = 0;

        for path in paths {
            if self
                .files
                .iter()
                .any(|existing| existing.path.eq_ignore_ascii_case(&path))
            {
                continue;
            }

            if let Ok(file) = self.repository.load(&path) {
                self.files.push(file);
                added += 1;
            }
        }

        added
    }

    fn is_mp3(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3"))
    }
}
