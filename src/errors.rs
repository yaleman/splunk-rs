//! Error things

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
