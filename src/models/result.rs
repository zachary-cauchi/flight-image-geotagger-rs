use std::{fmt::Display, num::TryFromIntError};

pub type GTResult<T> = Result<T, GTError>;

pub enum GTError {
    Parser,
    MissingData(String),
    InvalidData(String),
    Io(std::io::Error),
    Serde(String),
    Args(String),
    Exif(exif::Error),
    ImgHandling(img_parts::Error),
    Conversion(String),
}

impl From<img_parts::Error> for GTError {
    fn from(value: img_parts::Error) -> Self {
        Self::ImgHandling(value)
    }
}

impl From<TryFromIntError> for GTError {
    fn from(value: TryFromIntError) -> Self {
        Self::Conversion(value.to_string())
    }
}

impl From<exif::Error> for GTError {
    fn from(value: exif::Error) -> Self {
        Self::Exif(value)
    }
}

impl From<std::io::Error> for GTError {
    fn from(value: std::io::Error) -> Self {
        GTError::Io(value)
    }
}

impl From<serde_json::Error> for GTError {
    fn from(value: serde_json::Error) -> Self {
        GTError::Serde(value.to_string())
    }
}

impl Display for GTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parser => write!(f, "JSON Parser error."),
            Self::MissingData(e) => write!(f, "Missing data error: {e}"),
            Self::InvalidData(e) => write!(f, "Invalid data error: {e}"),
            Self::ImgHandling(e) => write!(f, "Image-handling error: {e}"),
            Self::Io(e) => write!(f, "IO parser error: {e}"),
            Self::Serde(e) => write!(f, "Serde error: {e}"),
            Self::Args(e) => write!(f, "CLI args config error: {e}"),
            Self::Exif(e) => write!(f, "Exif-related error: {e}"),
            Self::Conversion(e) => write!(f, "Data conversion error: {e}"),
        }
    }
}
