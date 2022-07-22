use std::fmt::{Debug, Display, Formatter};
use reqwest::Error;
use url::ParseError;

#[derive(Debug)]
pub enum ErrorKind {
    GenericError,

    UrlCannotBeParsed{
        message: String
    },
    StatusCodeFromUrl {
        url: String,
        status_code: u16,
        status: String,
    }
}

impl std::error::Error for ErrorKind {}

impl From<ParseError> for ErrorKind {
    fn from(parse_error: ParseError) -> Self {
        match parse_error {
            _ => ErrorKind::UrlCannotBeParsed {
                message: parse_error.to_string()
            }
        }
    }
}

impl From<reqwest::Error> for ErrorKind {
    fn from(reqwest_error: Error) -> Self {
        match reqwest_error {
            _ => ErrorKind::GenericError
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &*self {
            ErrorKind::GenericError => write!(f, "Something is off"),
            ErrorKind::UrlCannotBeParsed { message } => write!(f, "The url cannot be parsed: {}", message),
            ErrorKind::StatusCodeFromUrl { url, status_code, status } =>
                write!(f, "The url {} return status code {} {}", url, status_code, status),
        }
    }
}