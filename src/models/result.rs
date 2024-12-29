use std::fmt::Display;

pub type GTResult<T> = Result<T, GTError>;

pub enum GTError {
    Parser,
    Io(std::io::Error),
    Serde(String),
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
            Self::Io(e) => write!(f, "IO parser error: {e}"),
            Self::Serde(e) => write!(f, "Serde error: {e}"),
        }
    }
}
