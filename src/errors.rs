//! Error things

use std::{num::ParseIntError, time::SystemTimeError};

use reqwest::header::InvalidHeaderValue;

#[derive(Debug)]
/// Error messages and things
pub enum SplunkError {
    /// Generic errors not handled by something else
    Generic(String),
    /// We failed to create a search
    SearchCreationFailed(String),
    /// You haven't authenticated yet!
    NotAuthenticated,

    /// When `serde_json` doesn't like something you did
    SerdeError(serde_json::Error),

    /// When `reqwest` doesn't like something you did
    ReqwestError(reqwest::Error),

    /// Didn't select an Auth method
    NoAuthMethodSelected,

    /// Invalid Auth method selected
    InvalidAuthmethod(&'static str),
}

impl From<serde_json::Error> for SplunkError {
    fn from(value: serde_json::Error) -> Self {
        SplunkError::SerdeError(value)
    }
}

impl From<String> for SplunkError {
    fn from(value: String) -> Self {
        SplunkError::Generic(value)
    }
}

impl From<reqwest::Error> for SplunkError {
    fn from(value: reqwest::Error) -> Self {
        SplunkError::ReqwestError(value)
    }
}

impl From<serde_xml_rs::Error> for SplunkError {
    fn from(value: serde_xml_rs::Error) -> Self {
        SplunkError::Generic(format!("XML Parse Error: {}", value))
    }
}

impl From<InvalidHeaderValue> for SplunkError {
    fn from(value: InvalidHeaderValue) -> Self {
        SplunkError::Generic(format!("Invalid Header Value: {}", value))
    }
}

impl From<ParseIntError> for SplunkError {
    fn from(value: std::num::ParseIntError) -> Self {
        SplunkError::Generic(format!("Parse Int Error: {}", value))
    }
}

impl From<SystemTimeError> for SplunkError {
    fn from(value: std::time::SystemTimeError) -> Self {
        SplunkError::Generic(format!("System Time Error: {}", value))
    }
}
