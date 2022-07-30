use reqwest::Error;
use std::fmt::{Debug, Display, Formatter};
use url::ParseError;

#[derive(Debug)]
pub enum ErrorKind {
    GenericError,

    UrlCannotBeParsed {
        message: String,
    },
    StatusCodeFromUrl {
        url: String,
        status_code: u16,
        status: String,
    },

    CssSelectorError {
        message: String,
    },

    CannotFindAttribute {
        attribute_name: String,
    },

    CannotFindSelector {
        selector: String,
    },
    MongoDbError {
        mongodb_error: mongodb::error::Error,
    },
}

impl std::error::Error for ErrorKind {}

impl From<ParseError> for ErrorKind {
    fn from(parse_error: ParseError) -> Self {
        match parse_error {
            _ => ErrorKind::UrlCannotBeParsed {
                message: parse_error.to_string(),
            },
        }
    }
}

impl From<reqwest::Error> for ErrorKind {
    fn from(reqwest_error: Error) -> Self {
        match reqwest_error {
            _ => ErrorKind::GenericError,
        }
    }
}

impl From<mongodb::error::Error> for ErrorKind {
    fn from(mongo_error: mongodb::error::Error) -> Self {
        ErrorKind::MongoDbError {
            mongodb_error: mongo_error,
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &*self {
            ErrorKind::GenericError => write!(f, "Something is off"),
            ErrorKind::UrlCannotBeParsed { message } => {
                write!(
                    f,
                    "UrlCannotBeParsed: The url cannot be parsed: {}",
                    message
                )
            }
            ErrorKind::StatusCodeFromUrl {
                url,
                status_code,
                status,
            } => write!(
                f,
                "StatusCodeFromUrl: The url {} return status code {} {}",
                url, status_code, status
            ),
            ErrorKind::CssSelectorError { message } => {
                write!(f, "CssSelectorError: Error in css selector {}", message)
            }
            ErrorKind::CannotFindAttribute { attribute_name } => {
                write!(f, "CannotFindAttribute: {}", attribute_name)
            }
            ErrorKind::CannotFindSelector { selector } => {
                write!(f, "CannotFindSelector: {}", selector)
            }
            ErrorKind::MongoDbError { mongodb_error } => {
                write!(f, "MongoDbError: {:?}", mongodb_error)
            }
        }
    }
}
