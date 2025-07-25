use std::fmt;

#[derive(Debug)]
pub enum AppError {
    MissingArgument(String),
    FileNotFound(String),
    IoError(std::io::Error),
    CsvError(csv::Error),
    CsvAsyncError(csv_async::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingArgument(msg) => write!(f, "{msg}"),
            AppError::FileNotFound(path) => write!(f, "file {path} does not exist"),
            AppError::IoError(error) => write!(f, "IO error: {error}"),
            AppError::CsvError(error) => write!(f, "CSV error: {error}"),
            AppError::CsvAsyncError(error) => write!(f, "CSV async error: {error}"),
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

impl From<csv_async::Error> for AppError {
    fn from(err: csv_async::Error) -> Self {
        AppError::CsvAsyncError(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
