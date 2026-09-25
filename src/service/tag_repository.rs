use std::path::Path;

use crate::model::{AudioFile, TagData};

use super::error::AppResult;

pub trait TagRepository {
    fn load(&self, path: &Path) -> AppResult<AudioFile>;
    fn save_preserving_unknown_frames(&self, file: &AudioFile) -> AppResult<()>;
    fn write_clean_v23(&self, path: &Path, tags: &TagData) -> AppResult<()>;
}
