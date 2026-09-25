use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use id3::{
    frame::{Comment, Picture, PictureType},
    Tag, TagLike, Version,
};

use crate::model::{AudioFile, CoverArt, Diagnostics, TagData};

use super::{
    error::{AppError, AppResult},
    TagRepository,
};

#[derive(Default)]
pub struct Id3TagRepository;

impl Id3TagRepository {
    fn read_tag(path: &Path) -> Tag {
        Tag::read_from_path(path).unwrap_or_else(|_| Tag::new())
    }

    fn detect_id3_version(path: &Path) -> AppResult<String> {
        let mut file = File::open(path)?;
        let mut header = [0_u8; 4];
        let read = file.read(&mut header)?;

        if read < 4 || &header[0..3] != b"ID3" {
            return Ok("Sin ID3v2".to_owned());
        }

        Ok(match header[3] {
            2 => "ID3v2.2",
            3 => "ID3v2.3",
            4 => "ID3v2.4",
            other => return Ok(format!("ID3v2 desconocido ({other})")),
        }
        .to_owned())
    }

    fn detect_tail_tags(path: &Path) -> AppResult<(bool, bool)> {
        let mut file = File::open(path)?;
        let len = file.metadata()?.len();

        let mut has_id3v1 = false;
        if len >= 128 {
            file.seek(SeekFrom::End(-128))?;
            let mut marker = [0_u8; 3];
            file.read_exact(&mut marker)?;
            has_id3v1 = &marker == b"TAG";
        }

        let ape_footer_offset = if has_id3v1 { 160 } else { 32 };
        let mut has_apev2 = false;
        if len >= ape_footer_offset {
            file.seek(SeekFrom::End(-(ape_footer_offset as i64)))?;
            let mut marker = [0_u8; 8];
            file.read_exact(&mut marker)?;
            has_apev2 = &marker == b"APETAGEX";
        }

        Ok((has_id3v1, has_apev2))
    }

    fn from_tag(tag: &Tag) -> TagData {
        let comment = tag
            .comments()
            .next()
            .map(|value| value.text.clone())
            .unwrap_or_default();

        let cover = tag.pictures().next().map(|picture| CoverArt {
            mime_type: picture.mime_type.clone(),
            description: picture.description.clone(),
            data: picture.data.clone(),
        });

        TagData {
            title: tag.title().unwrap_or_default().to_owned(),
            artist: tag.artist().unwrap_or_default().to_owned(),
            album: tag.album().unwrap_or_default().to_owned(),
            genre: tag.genre().unwrap_or_default().to_owned(),
            year: tag.year().map(|value| value.to_string()).unwrap_or_default(),
            track: tag.track().map(|value| value.to_string()).unwrap_or_default(),
            disc: tag.disc().map(|value| value.to_string()).unwrap_or_default(),
            comment,
            cover,
        }
    }

    fn set_text_field(tag: &mut Tag, id: &str, value: &str, setter: impl FnOnce(&mut Tag, &str)) {
        if value.trim().is_empty() {
            tag.remove(id);
        } else {
            setter(tag, value.trim());
        }
    }

    fn apply_common_fields(tag: &mut Tag, data: &TagData) -> AppResult<()> {
        Self::set_text_field(tag, "TIT2", &data.title, |tag, value| tag.set_title(value));
        Self::set_text_field(tag, "TPE1", &data.artist, |tag, value| tag.set_artist(value));
        Self::set_text_field(tag, "TALB", &data.album, |tag, value| tag.set_album(value));
        Self::set_text_field(tag, "TCON", &data.genre, |tag, value| tag.set_genre(value));

        tag.remove("TYER");
        tag.remove("TDRC");
        if !data.year.trim().is_empty() {
            let year = data
                .year
                .trim()
                .parse::<i32>()
                .map_err(|_| AppError::Message("El año debe ser un número entero".to_owned()))?;
            tag.set_year(year);
        }

        tag.remove("TRCK");
        if !data.track.trim().is_empty() {
            let track = data
                .track
                .trim()
                .parse::<u32>()
                .map_err(|_| AppError::Message("La pista debe ser un número entero".to_owned()))?;
            tag.set_track(track);
        }

        tag.remove("TPOS");
        if !data.disc.trim().is_empty() {
            let disc = data
                .disc
                .trim()
                .parse::<u32>()
                .map_err(|_| AppError::Message("El disco debe ser un número entero".to_owned()))?;
            tag.set_disc(disc);
        }

        tag.remove("COMM");
        if !data.comment.trim().is_empty() {
            tag.add_frame(Comment {
                lang: "spa".to_owned(),
                description: String::new(),
                text: data.comment.trim().to_owned(),
            });
        }

        tag.remove("APIC");
        if let Some(cover) = &data.cover {
            tag.add_frame(Picture {
                mime_type: cover.mime_type.clone(),
                picture_type: PictureType::CoverFront,
                description: cover.description.clone(),
                data: cover.data.clone(),
            });
        }

        Ok(())
    }

    fn tlen(tag: &Tag) -> Option<String> {
        tag.get("TLEN")
            .and_then(|frame| frame.content().text())
            .map(ToOwned::to_owned)
    }
}

impl TagRepository for Id3TagRepository {
    fn load(&self, path: &Path) -> AppResult<AudioFile> {
        let tag = Self::read_tag(path);
        let id3_version = Self::detect_id3_version(path)?;
        let (has_id3v1, has_apev2) = Self::detect_tail_tags(path)?;
        let raw_tlen = Self::tlen(&tag);
        let tags = Self::from_tag(&tag);

        let cover_description = tags.cover.as_ref().map(|cover| {
            format!(
                "{} · {:.1} KiB",
                cover.mime_type,
                cover.data.len() as f32 / 1024.0
            )
        });

        Ok(AudioFile {
            path: path.to_path_buf(),
            tags,
            diagnostics: Diagnostics {
                id3_version,
                raw_tlen,
                has_id3v1,
                has_apev2,
                cover_description,
            },
            dirty: false,
        })
    }

    fn save_preserving_unknown_frames(&self, file: &AudioFile) -> AppResult<()> {
        let mut tag = Self::read_tag(&file.path);
        Self::apply_common_fields(&mut tag, &file.tags)?;
        tag.write_to_path(&file.path, Version::Id3v23)?;
        Ok(())
    }

    fn write_clean_v23(&self, path: &Path, tags: &TagData) -> AppResult<()> {
        let mut tag = Tag::new();
        Self::apply_common_fields(&mut tag, tags)?;

        tag.remove("TLEN");
        tag.write_to_path(path, Version::Id3v23)?;
        Ok(())
    }
}
