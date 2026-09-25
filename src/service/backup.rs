use std::{
    fs,
    path::{Path, PathBuf},
};

use super::error::{AppError, AppResult};

pub trait BackupService {
    fn backup(&self, source: &Path) -> AppResult<Option<PathBuf>>;
}

#[derive(Default)]
pub struct FileBackupService;

impl BackupService for FileBackupService {
    fn backup(&self, source: &Path) -> AppResult<Option<PathBuf>> {
        let parent = source
            .parent()
            .ok_or_else(|| AppError::Message("El archivo no tiene carpeta padre".to_owned()))?;

        let backup_dir = parent.join(".nq4-backup");
        fs::create_dir_all(&backup_dir)?;

        let file_name = source
            .file_name()
            .ok_or_else(|| AppError::Message("Nombre de archivo inválido".to_owned()))?;

        let target = backup_dir.join(file_name);

        if target.exists() {
            return Ok(None);
        }

        fs::copy(source, &target)?;
        Ok(Some(target))
    }
}
