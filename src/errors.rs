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
}
