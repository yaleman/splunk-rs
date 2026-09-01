//! Server Config Declaration

use std::{env, str::FromStr};

use reqwest::{header::HeaderMap, Client, Response, Url};
use serde::{Deserialize, Serialize};

use crate::{errors::SplunkError, models::AuthenticationMethod};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// What we're going to use to connect to the server.
///
/// Built via [`ServerConfigBuilder`] - the connection URL is resolved once, at build time,
/// from the hostname/port/scheme it was given. It's stored privately here and exposed only
/// through [`ServerConfig::url`], so it can't drift after the fact - to point at a different
/// server, build a new one.
pub struct ServerConfig {
    url: Url,
    pub(crate) verify_tls: bool,
    pub(crate) auth_method: AuthenticationMethod,
    connection_timeout: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        // The default builder only ever feeds set_host/set_scheme/set_port known-good,
        // hardcoded values, so building it can't actually fail.
        #[allow(clippy::expect_used)]
        ServerConfigBuilder::default()
            .build()
            .expect("the default ServerConfigBuilder must always produce a valid ServerConfig")
    }
}

impl ServerConfig {
    /// Start building a config pointed at `hostname`
    pub fn builder(hostname: impl Into<String>) -> ServerConfigBuilder {
        ServerConfigBuilder::new(hostname)
    }

    /// The base URL this config connects to (scheme + host + port). This is fixed at build
    /// time - to point at a different server, build a new [`ServerConfig`] via
    /// [`ServerConfigBuilder`].
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Resolve a request URL for a given endpoint, joined onto this config's base
    /// [`ServerConfig::url`].
    /// ```
    /// use splunk::server_config::ServerConfig;
    ///
    /// let config = ServerConfig::builder("localhost")
    ///     .with_port(8088)
    ///     .build()
    ///     .expect("Failed to build config");
    /// assert_eq!(
    ///     config.get_url("/hello").expect("Failed to get URL").as_str(),
    ///     "https://localhost:8088/hello"
    /// );
    /// ```
    pub fn get_url(&self, endpoint: &str) -> Result<Url, SplunkError> {
        self.url.join(endpoint).map_err(|e| {
            SplunkError::Generic(format!(
                "Failed to build URL for endpoint {endpoint:?}: {e:?}"
            ))
        })
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
    pub async fn do_get(&self, endpoint: &str) -> Result<Response, SplunkError> {
        let headers = HeaderMap::new();
        self.do_get_with_headers(endpoint, headers).await
    }

    /// make a get request to a given endpoint and set the headers
    pub async fn do_get_with_headers(
        &self,
        endpoint: &str,
        add_headers: HeaderMap,
    ) -> Result<Response, SplunkError> {
        let request = Client::new().get(self.get_url(endpoint)?);

        let mut headers = HeaderMap::new();

        // apply the supplied_headers
        for (key, value) in add_headers.into_iter() {
            if let Some(key_name) = key {
                headers.insert(key_name, value);
            }
        }

        let request = match &self.auth_method {
            AuthenticationMethod::Token { token } => {
                headers.insert("Authorization", format!("Splunk {}", token).parse()?);
                request.headers(headers)
            }
            AuthenticationMethod::Basic { username, password } => {
                request.basic_auth(username, Some(password))
            }
            #[allow(clippy::todo)]
            _ => todo!("haven't handled all the things yet"),
        };

        // eprintln!("{:#?}", request);
        request.send().await.map_err(SplunkError::from)
    }

    /// Grabs a [ServerConfig] from environment variables
    pub fn try_from_env(configtype: ServerConfigType) -> Result<ServerConfig, SplunkError> {
        ServerConfigBuilder::try_from_env(configtype)?.build()
    }
}

/// Builds a [`ServerConfig`]. Set the hostname, port, TLS and auth options here, then call
/// [`ServerConfigBuilder::build`] to resolve them into a [`ServerConfig`] - after that the
/// connection URL is fixed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfigBuilder {
    hostname: String,
    port: u16,
    verify_tls: bool,
    use_tls: bool,
    auth_method: AuthenticationMethod,
    connection_timeout: u16,
}

impl Default for ServerConfigBuilder {
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

impl ServerConfigBuilder {
    /// Point at a server
    pub fn new(hostname: impl Into<String>) -> Self {
        Self {
            hostname: hostname.into(),
            ..Default::default()
        }
    }

    /// Resolve this builder into a [`ServerConfig`], baking the hostname/port/scheme into a
    /// single immutable URL.
    pub fn build(self) -> Result<ServerConfig, SplunkError> {
        // Start from an already-"special" scheme (http/https/ws/wss/ftp/file) - `Url::set_scheme`
        // refuses to cross the special/non-special boundary, so a placeholder like `a://a.a`
        // can never be switched to `http`/`https`.
        let mut url = Url::from_str("http://a.a")
            .map_err(|e| SplunkError::Generic(format!("Failed to build URL: {e:?}")))?;
        url.set_host(Some(&self.hostname))
            .map_err(|e| SplunkError::Generic(format!("Failed to set host: {e:?}")))?;

        match self.use_tls {
            true => url
                .set_scheme("https")
                .map_err(|_| SplunkError::Generic("Could not set scheme".to_string()))?,
            false => url
                .set_scheme("http")
                .map_err(|_| SplunkError::Generic("Could not set scheme".to_string()))?,
        };

        if (self.verify_tls && self.port != 443) || (!self.verify_tls && self.port != 80) {
            url.set_port(Some(self.port))
                .map_err(|_| SplunkError::Generic("Could not set port".to_string()))?;
        }

        Ok(ServerConfig {
            url,
            verify_tls: self.verify_tls,
            auth_method: self.auth_method,
            connection_timeout: self.connection_timeout,
        })
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
    pub fn with_username_password(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.auth_method = AuthenticationMethod::Basic {
            username: username.into(),
            password: password.into(),
        };
        self
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the hostname
    pub fn with_hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = hostname.into();
        self
    }

    /// Do we verify TLS on send?
    pub fn with_verify_tls(mut self, verify_tls: bool) -> Self {
        self.verify_tls = verify_tls;
        self
    }

    /// Grabs a [ServerConfigBuilder] from environment variables, ready for further overrides
    /// before calling [`ServerConfigBuilder::build`].
    pub fn try_from_env(configtype: ServerConfigType) -> Result<ServerConfigBuilder, SplunkError> {
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
        let port: u16 = port.parse::<u16>()?;

        let config = ServerConfigBuilder::new(hostname).with_port(port);
        let config = match configtype {
            ServerConfigType::Hec => {
                let Ok(token) = env::var(format!("{env_prefix}TOKEN")) else {
                    let error = SplunkError::Generic(format!(
                        "Please ensure env var {env_prefix}TOKEN is set"
                    ));
                    return Err(error);
                };
                config.with_token(token)
            }

            ServerConfigType::Api => {
                let Ok(username) = env::var("SPLUNK_USERNAME") else {
                    let error = SplunkError::Generic(format!(
                        "Please ensure env var {env_prefix}USERNAME is set"
                    ));
                    return Err(error);
                };
                let Ok(password) = env::var("SPLUNK_PASSWORD") else {
                    let error = SplunkError::Generic(format!(
                        "Please ensure env var {env_prefix}PASSWORD is set"
                    ));
                    return Err(error);
                };

                config.with_username_password(username, password)
            }
        };
        Ok(config)
    }
}

/// This is just used in get_serverconfig so you can say "I need a HEC or I need an API one!"
#[derive(Copy, Clone, Debug)]
pub enum ServerConfigType {
    /// You're using HTTP Event Collector - looks for SPLUNK_HEC_*
    Hec,
    /// You're using API Endpoints - looks for SPLUNK_API_*
    Api,
}
