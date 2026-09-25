#[derive(Clone, Debug)]
pub struct NormalizationOptions {
    pub create_backup: bool,
    pub strip_id3v1: bool,
    pub strip_apev2: bool,
    pub normalize_cover: bool,
    pub max_cover_size: u32,
    pub jpeg_quality: u8,
}

impl Default for NormalizationOptions {
    fn default() -> Self {
        Self {
            create_backup: true,
            strip_id3v1: true,
            strip_apev2: true,
            normalize_cover: true,
            max_cover_size: 600,
            jpeg_quality: 88,
        }
    }
}
