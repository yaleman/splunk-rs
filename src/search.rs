//! Want to search things? Here's your place.


/// Client for splunk enterprise/cloud API things, login, search, manipulate config items etc.
pub struct SplunkClient {
    server: String,
    port: u16,
    validate_ssl: bool,
}

impl Default for SplunkClient {
    fn default() -> Self {
        Self {
            server: "localhost".to_string(),
            port: 8089,
            validate_ssl: true,
        }
    }
}

impl SplunkClient {
    /// Login and establish the session
    pub fn login() -> Result<(), String> {
        unimplemented!();
    }
}