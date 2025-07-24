use std::fmt;

#[derive(Debug)]
pub enum AppError {
    MissingArgument(String),
    FileNotFound(String),
    IoError(std::io::Error),
    CsvError(csv::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingArgument(msg) => write!(f, "{}", msg),
            AppError::FileNotFound(path) => write!(f, "file {} does not exist", path),
            AppError::IoError(error) => write!(f, "IO error: {}", error),
            AppError::CsvError(error) => write!(f, "CSV error: {}", error),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<csv::Error> for AppError {
    fn from(err: csv::Error) -> Self {
        AppError::CsvError(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
