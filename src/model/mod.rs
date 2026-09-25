mod audio_file;
mod normalization;
mod tag_data;

pub use audio_file::{AudioFile, Diagnostics};
pub use normalization::{NormalizationOptions, NORMALIZED_FOLDER_NAME};
pub use tag_data::{CoverArt, TagData};
