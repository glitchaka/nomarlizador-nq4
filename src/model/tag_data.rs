#[derive(Clone, Debug, Default)]
pub struct CoverArt {
    pub mime_type: String,
    pub description: String,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct TagData {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub year: String,
    pub track: String,
    pub disc: String,
    pub comment: String,
    pub cover: Option<CoverArt>,
}
