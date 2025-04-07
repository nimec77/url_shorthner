use std::fmt::{self, Display};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum AppError {
    NotFound,
    UrlParseError,
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound => write!(f, "Not found"),
            AppError::UrlParseError => write!(f, "URL parse error"),
        }
    }
}


