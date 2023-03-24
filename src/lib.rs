//! Placeholder for now.

use std::str::FromStr;

use reqwest::Url;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate lazy_static;

#[allow(unused_imports)]
#[macro_use]
extern crate tokio;

#[cfg(feature = "hec")]
pub mod hec;
#[cfg(feature = "search")]
pub mod search;

#[cfg(debug_assertions)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub hostname: String,
    pub port: u16,
    validate_ssl: bool,
    use_tls: bool,
    username: Option<String>,
    password: Option<String>
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            port: 8089,
            validate_ssl: true,
            use_tls: true,
            username: None,
            password: None,
        }
    }
}

impl ServerConfig {
    /// build a url based on the server/endpoint
    /// ```
    /// use std::str::FromStr;
    /// use reqwest::Url;
    /// use splunk::hec::HecClient;
    ///
    /// let client = HecClient::new();
    /// let expected_response = Url::from_str("https://localhost:8088/hello").unwrap();
    /// assert_eq!(client.serverconfig.get_url("/hello").unwrap(), expected_response);
    /// ```
    pub fn get_url(&self, endpoint: &str) -> Result<Url, String> {
        let mut result = String::new();

        result.push_str(match self.use_tls {
            true => "https",
            false => "http",
        });

        result.push_str("://");
        result.push_str(&self.hostname);
        if (self.use_tls && self.port != 443) || (!self.use_tls && self.port != 80) {
            result.push_str(&format!(":{}", self.port));
        }
        result.push_str(endpoint);
        Url::from_str(&result).map_err(|e| format!("{e:?}"))
    }
}
