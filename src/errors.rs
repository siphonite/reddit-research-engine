use std::fmt;

pub enum AppError {
    InvalidInput(String),
    ExternalService(String),
    Io(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidInput(msg) => write!(f, "{}", msg),
            AppError::ExternalService(msg) => write!(f, "{}", msg),
            AppError::Io(msg) => write!(f, "{}", msg),
        }
    }
}
