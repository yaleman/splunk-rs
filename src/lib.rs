//! Placeholder for now.

use std::env;
use std::error::Error;
use std::str::FromStr;

use reqwest::header::HeaderMap;
use reqwest::{Response, Url};
use search::AuthenticationMethod;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate lazy_static;

#[allow(unused_imports)]
#[macro_use]
extern crate tokio;

#[cfg(feature = "hec")]
pub mod hec;
#[cfg(feature = "search")]
#[macro_use]
pub mod search;

#[cfg(debug_assertions)]
mod tests;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub hostname: String,
    pub port: u16,
    validate_ssl: bool,
    verify_tls: bool,
    auth_method: AuthenticationMethod,
    connection_timeout: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            port: 8089,
            validate_ssl: true,
            verify_tls: true,
            auth_method: AuthenticationMethod::Unknown,
            connection_timeout: 30,
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
    /// let client = HecClient::new("token", "localhost");
    /// let expected_response = Url::from_str("https://localhost:8088/hello").unwrap();
    /// assert_eq!(client.serverconfig.get_url("/hello").unwrap(), expected_response);
    /// ```
    pub fn get_url(&self, endpoint: &str) -> Result<Url, String> {
        let mut result = String::new();

        result.push_str(match self.verify_tls {
            true => "https",
            false => "http",
        });

        result.push_str("://");
        result.push_str(&self.hostname);
        if (self.verify_tls && self.port != 443) || (!self.verify_tls && self.port != 80) {
            result.push_str(&format!(":{}", self.port));
        }
        result.push_str(endpoint);
        Url::from_str(&result).map_err(|e| format!("{e:?}"))
    }

    pub fn new(hostname: String) -> Self {
        Self {
            hostname,
            ..Default::default()
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.auth_method = AuthenticationMethod::Token { token };
        self
    }

    /// Get the token from the auth method, if it exists
    pub fn token(&self) -> Option<String> {
        match &self.auth_method {
            AuthenticationMethod::Basic {
                username: _,
                password,
            } => Some(password.to_owned()),
            AuthenticationMethod::Token { token } => Some(token.to_owned()),
            AuthenticationMethod::Unknown => None,
        }
    }

    pub fn with_username_password(mut self, username: String, password: String) -> Self {
        self.auth_method = AuthenticationMethod::Basic { username, password };
        self
    }

    pub async fn do_get(
        &self,
        _endpoint: &str,
        _headers: HeaderMap,
    ) -> Result<Response, Box<dyn Error>> {
        todo!();
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the hostname
    pub fn with_hostname(mut self, hostname: String) -> Self {
        self.hostname = hostname;
        self
    }

    /// Do we verify TLS on send?
    pub fn with_verify_tls(mut self, verify_tls: bool) -> Self {
        self.verify_tls = verify_tls;
        self
    }
}

/// This is just used in get_serverconfig so you can say "I need a HEC or I need an API one!"
pub enum ServerConfigType {
    Hec,
    Api,
}

pub fn get_serverconfig(configtype: ServerConfigType) -> Result<ServerConfig, String> {
    let env_prefix = match configtype {
        ServerConfigType::Hec => "SPLUNK_HEC_",
        ServerConfigType::Api => "SPLUNK_API_",
    };

    let hostname = match env::var(format!("{env_prefix}HOSTNAME")) {
        Ok(val) => val,
        Err(_) => {
            let error = format!("Please ensure env var {env_prefix}HOSTNAME is set");
            eprintln!("{}", error);
            return Err(error);
        }
    };
    let port = match env::var(format!("{env_prefix}PORT")) {
        Ok(val) => val,
        Err(_) => 8089.to_string(),
    };
    let port: u16 = port.parse::<u16>().unwrap();

    let config = ServerConfig::new(hostname).with_port(port);
    let config = match configtype {
        ServerConfigType::Hec => {
            let token = env::var(format!("{env_prefix}TOKEN"))
                .expect("Couldn't get SPLUNK_HEC_TOKEN env var");
            config.with_token(token)
        }
        ServerConfigType::Api => config.with_username_password(
            env::var("SPLUNK_USERNAME").expect("Couldn't get SPLUNK_USERNAME env var!"),
            env::var("SPLUNK_PASSWORD").expect("Couldn't get SPLUNK_PASSWORD env var!"),
        ),
    };
    Ok(config)
}
