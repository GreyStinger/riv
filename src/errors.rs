use thiserror::Error;

#[derive(Debug, Error)]
pub enum RviError {
    #[error("Unable to create window")]
    WindowError(#[from] winit::error::OsError),
    #[error("An error occurred while processing the image")]
    ImageError(#[from] image::ImageError),
    #[error("And error occurred while loading the image")]
    IoError(#[from] std::io::Error),
    #[error("Unable to create new pixels instance")]
    PixelsError(#[from] pixels::Error),
    #[error("Cannot find primary monitor")]
    NoPrimaryMonitor,
}

pub type Result<T> = std::result::Result<T, RviError>;
