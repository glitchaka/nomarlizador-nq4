use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Error de archivo: {0}")]
    Io(#[from] io::Error),

    #[error("Error ID3: {0}")]
    Id3(#[from] id3::Error),

    #[error("Error de imagen: {0}")]
    Image(#[from] image::ImageError),

    #[error("{0}")]
    Message(String),
}

pub type AppResult<T> = Result<T, AppError>;
