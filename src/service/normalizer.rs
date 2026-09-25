use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use image::{codecs::jpeg::JpegEncoder, DynamicImage};

use crate::model::{AudioFile, CoverArt, NormalizationOptions, TagData};

use super::{
    error::{AppError, AppResult},
    BackupService, TagRepository,
};

pub trait AudioNormalizer {
    fn normalize(
        &self,
        file: &AudioFile,
        options: &NormalizationOptions,
        repository: &dyn TagRepository,
        backup: &dyn BackupService,
    ) -> AppResult<()>;
}

#[derive(Default)]
pub struct IpodSafeNormalizer;

impl IpodSafeNormalizer {
    fn normalize_cover(data: &TagData, options: &NormalizationOptions) -> AppResult<TagData> {
        let mut normalized = data.clone();

        if !options.normalize_cover {
            return Ok(normalized);
        }

        let Some(cover) = &data.cover else {
            return Ok(normalized);
        };

        let decoded = image::load_from_memory(&cover.data)?;
        let resized = if decoded.width() > options.max_cover_size
            || decoded.height() > options.max_cover_size
        {
            decoded.thumbnail(options.max_cover_size, options.max_cover_size)
        } else {
            decoded
        };

        let rgb = DynamicImage::ImageRgb8(resized.to_rgb8());
        let mut output = Vec::new();
        JpegEncoder::new_with_quality(&mut output, options.jpeg_quality)
            .encode_image(&rgb)?;

        normalized.cover = Some(CoverArt {
            mime_type: "image/jpeg".to_owned(),
            description: cover.description.clone(),
            data: output,
        });

        Ok(normalized)
    }

    fn strip_id3v1(path: &Path) -> AppResult<()> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let len = file.metadata()?.len();

        if len < 128 {
            return Ok(());
        }

        file.seek(SeekFrom::End(-128))?;
        let mut marker = [0_u8; 3];
        file.read_exact(&mut marker)?;

        if &marker == b"TAG" {
            file.set_len(len - 128)?;
        }

        Ok(())
    }

    fn strip_apev2(path: &Path) -> AppResult<()> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let len = file.metadata()?.len();

        if len < 32 {
            return Ok(());
        }

        file.seek(SeekFrom::End(-32))?;
        let mut footer = [0_u8; 32];
        file.read_exact(&mut footer)?;

        if &footer[0..8] != b"APETAGEX" {
            return Ok(());
        }

        let size = u32::from_le_bytes(
            footer[12..16]
                .try_into()
                .map_err(|_| AppError::Message("Footer APEv2 inválido".to_owned()))?,
        ) as u64;

        if size < 32 || size > len {
            return Err(AppError::Message(
                "Se detectó APEv2, pero su tamaño es inválido; no se modificó".to_owned(),
            ));
        }

        let mut truncate_to = len - size;

        // Si existe un header APEv2 adicional inmediatamente antes del bloque,
        // también se elimina. El tamaño declarado por APE excluye ese header.
        if truncate_to >= 32 {
            file.seek(SeekFrom::Start(truncate_to - 32))?;
            let mut possible_header = [0_u8; 8];
            file.read_exact(&mut possible_header)?;
            if &possible_header == b"APETAGEX" {
                truncate_to -= 32;
            }
        }

        file.set_len(truncate_to)?;
        Ok(())
    }
}

impl AudioNormalizer for IpodSafeNormalizer {
    fn normalize(
        &self,
        file: &AudioFile,
        options: &NormalizationOptions,
        repository: &dyn TagRepository,
        backup: &dyn BackupService,
    ) -> AppResult<()> {
        if options.create_backup {
            backup.backup(&file.path)?;
        }

        if options.strip_id3v1 {
            Self::strip_id3v1(&file.path)?;
        }

        if options.strip_apev2 {
            Self::strip_apev2(&file.path)?;
        }

        let tags = Self::normalize_cover(&file.tags, options)?;
        repository.write_clean_v23(&file.path, &tags)
    }
}
