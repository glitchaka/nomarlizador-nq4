use std::path::PathBuf;

use super::TagData;

#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    pub id3_version: String,
    pub raw_tlen: Option<String>,
    pub has_id3v1: bool,
    pub has_apev2: bool,
    pub cover_description: Option<String>,
}

impl Diagnostics {
    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.id3_version == "ID3v2.4" {
            warnings.push("ID3v2.4: se normalizará a ID3v2.3".to_owned());
        } else if self.id3_version == "Sin ID3v2" {
            warnings.push("El archivo no tiene ID3v2; se creará una etiqueta".to_owned());
        }

        if let Some(tlen) = &self.raw_tlen {
            let detail = match tlen.parse::<u64>() {
                Ok(value) if value < 1_000 => format!("TLEN sospechoso: {tlen} ms"),
                _ => format!("TLEN presente: {tlen}"),
            };
            warnings.push(detail);
        }

        if self.has_id3v1 {
            warnings.push("Contiene ID3v1 al final del archivo".to_owned());
        }

        if self.has_apev2 {
            warnings.push("Contiene APEv2 al final del archivo".to_owned());
        }

        warnings
    }
}

#[derive(Clone, Debug)]
pub struct AudioFile {
    pub path: PathBuf,
    pub tags: TagData,
    pub diagnostics: Diagnostics,
}

impl AudioFile {
    pub fn display_name(&self) -> String {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("MP3")
            .to_owned()
    }
}
