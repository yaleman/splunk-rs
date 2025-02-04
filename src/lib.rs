#![warn(missing_docs)]

//! A start on implementing Splunk-SDK-like-things

use std::env;
use std::str::FromStr;

use client::AuthenticationMethod;
use reqwest::header::HeaderMap;
use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize};

use crate::errors::SplunkError;

#[allow(unused_imports)]
#[macro_use]
extern crate tokio;
pub mod errors;
pub mod hec;
#[macro_use]
pub mod search;

pub mod client;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Serialize, Deserialize)]
/// What we're going to use to connect to the server
pub struct ServerConfig {
    /// Server hostname - just something like example.com (or an IP, if you're like that)
    pub hostname: String,
    /// Port - defaults to 8089
    pub port: u16,
    verify_tls: bool,
    use_tls: bool,
    auth_method: AuthenticationMethod,
    connection_timeout: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            port: 8089,
            verify_tls: true,
            use_tls: true,
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
    pub fn get_url(&self, endpoint: impl ToString) -> Result<Url, String> {
        let mut result = String::new();

        result.push_str(match self.use_tls {
            true => "https",
            false => "http",
        });

        result.push_str("://");
        result.push_str(&self.hostname);
        if (self.verify_tls && self.port != 443) || (!self.verify_tls && self.port != 80) {
            result.push_str(&format!(":{}", self.port));
        }
        result.push_str(&endpoint.to_string());
        Url::from_str(&result).map_err(|e| format!("{e:?}"))
    }

    /// Point at a server
    pub fn new(hostname: String) -> Self {
        Self {
            hostname,
            ..Default::default()
        }
    }

    /// Set the authentication method to token and set the token
    pub fn with_token(mut self, token: String) -> Self {
        self.auth_method = AuthenticationMethod::Token { token };
        self
    }

    /// Are we using https?
    pub fn use_tls(mut self, setting: bool) -> Self {
        self.use_tls = setting;
        self
    }

    /// Set the authentication method to basic and set the credentials
    pub fn with_username_password(mut self, username: String, password: String) -> Self {
        self.auth_method = AuthenticationMethod::Basic { username, password };
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
            AuthenticationMethod::Cookie { .. } => None,
        }
    }

    /// make a get request to a given endpoint
    pub async fn do_get(&self, endpoint: &str) -> Result<Response, String> {
        let headers = HeaderMap::new();
        self.do_get_with_headers(endpoint, headers).await
    }

    /// make a get request to a given endpoint and set the headers
    pub async fn do_get_with_headers(
        &self,
        endpoint: &str,
        add_headers: HeaderMap,
    ) -> Result<Response, String> {
        let request = Client::new().get(self.get_url(endpoint).unwrap());

        let mut headers = HeaderMap::new();

        // apply the supplied_headers
        add_headers.into_iter().for_each(|(key, val)| {
            headers.insert(key.unwrap(), val);
        });

        let request = match &self.auth_method {
            AuthenticationMethod::Token { token } => {
                headers.insert(
                    "Authorization",
                    format!("Splunk {}", token).parse().unwrap(),
                );
                request.headers(headers)
            }
            AuthenticationMethod::Basic { username, password } => {
                request.basic_auth(username, Some(password))
            }
            _ => todo!("haven't handled all the things yet"),
        };

        // eprintln!("{:#?}", request);
        request.send().await.map_err(|e| format!("{e:?}"))
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

    /// Grabs a [ServerConfig] from environment variables
    pub fn try_from_env(configtype: ServerConfigType) -> Result<ServerConfig, SplunkError> {
        let env_prefix = match configtype {
            ServerConfigType::Hec => "SPLUNK_HEC_",
            ServerConfigType::Api => "SPLUNK_API_",
        };

        let hostname = match env::var(format!("{env_prefix}HOSTNAME")) {
            Ok(val) => val,
            Err(_) => {
                let error = SplunkError::Generic(format!(
                    "Please ensure env var {env_prefix}HOSTNAME is set"
                ));
                eprintln!("{:?}", error);
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
}

/// This is just used in get_serverconfig so you can say "I need a HEC or I need an API one!"
pub enum ServerConfigType {
    /// You're using HTTP Event Collector - looks for SPLUNK_HEC_*
    Hec,
    /// You're using API Endpoints - looks for SPLUNK_API_*
    Api,
}
