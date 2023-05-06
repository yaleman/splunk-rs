//! Want to search things? Here's your place.

use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
// use serde_json::Value;
use std::collections::HashMap;

use crate::ServerConfig;

#[macro_use]
pub mod searchjob;
pub mod kvstore;


pub use searchjob::{SearchJob, SearchResult};

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The current auth method for the search client
pub enum AuthenticationMethod {
    /// Basic auth
    Basic {
        /// username
        username: String,
        ///password
        password: String,
    },
    /// Token auth
    Token {
        /// token auth
        token: String,
    },
    /// Cookie based
    Cookie {
        /// cookie store
        cookie: HashMap<String, String>,
    },
    /// we haven't set it yet
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
/// the current auth mode - you can auth with username/password then get a cookie and go from there
pub enum AuthenticatedSessionMode {
    /// cookie auth
    Cookie {
        /// cookie value
        value: HashMap<String, String>,
    },
    /// token auth
    Token {
        /// the token
        value: String,
    },
    /// we haven't set it yet
    Unset,
}

#[derive(Debug, Deserialize, Serialize)]
/// Client for splunk enterprise/cloud API things, login, search, manipulate config items etc.
pub struct SplunkClient {
    #[serde(flatten)]
    /// server configuration object
    pub serverconfig: ServerConfig,
    /// what mode we're using for authentication (token, cookie etc)
    pub auth_session_mode: AuthenticatedSessionMode,
    #[serde(skip)]
    client: Client,
}

impl Default for SplunkClient {
    fn default() -> Self {
        Self {
            serverconfig: ServerConfig::default(),
            auth_session_mode: AuthenticatedSessionMode::Unset,
            client: Client::new(),
        }
    }
}

impl SplunkClient {
    /// set the config on build
    pub fn with_config(self, serverconfig: ServerConfig) -> Self {
        Self {
            serverconfig,
            ..self
        }
    }
    /// Make a POST request
    pub async fn do_post(
        &mut self,
        endpoint: &str,
        payload: HashMap<&str, String>,
    ) -> Result<Response, String> {
        let req = self
            .client
            .post(self.serverconfig.get_url(endpoint).unwrap())
            .form(&payload);

        let req = match &self.serverconfig.auth_method {
            AuthenticationMethod::Basic { username, password } => {
                req.basic_auth(username, Some(password))
            }
            AuthenticationMethod::Token { token } => {
                req.header("Authorization", format!("Splunk {}", token))
            }
            AuthenticationMethod::Unknown => todo!(),
            // TODO: handle cookie auth for posts?
            AuthenticationMethod::Cookie { cookie: _ } => todo!(),
        };

        eprintln!("About to post this: {req:#?}");
        eprintln!("About to post this: {:#?}", payload);

        req.send().await.map_err(|e| format!("{e:?}"))
    }

    /// Make a GET request, tries to pass the authentication automagically
    pub async fn do_get(&mut self, endpoint: &str) -> Result<Response, String> {
        let request = self
            .client
            .get(self.serverconfig.get_url(endpoint).unwrap());

        let request = match &self.auth_session_mode {
            AuthenticatedSessionMode::Token { value } => {
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Authorization",
                    format!("Splunk {}", value).parse().unwrap(),
                );
                request.headers(headers)
            }
            AuthenticatedSessionMode::Cookie { value: _ } => todo!(),
            AuthenticatedSessionMode::Unset => todo!(),
        };

        // eprintln!("{:#?}", request);
        request.send().await.map_err(|e| format!("{e:?}"))
    }

    /// Login and establish the session
    pub async fn login(&mut self) -> Result<(), String> {
        let endpoint = "/services/auth/login";

        let mut payload: HashMap<&str, String> = HashMap::new();

        match &self.serverconfig.auth_method {
            AuthenticationMethod::Basic { username, password } => {
                // request.basic_auth(username, Some(password)),
                payload.insert("username", username.to_owned());
                payload.insert("password", password.to_owned());
            }
            // AuthenticationMethod::Token { token } => todo!(),
            AuthenticationMethod::Unknown => panic!("Please specify an auth method!"),
            _ => unimplemented!("Token mode isn't supported!"),
        };

        let request = self.do_post(endpoint, payload).await?;

        // eprintln!("Response: {:#?}", request.headers());
        let body = request.text().await.unwrap();
        let res: serde_json::Value = serde_xml_rs::from_str(&body)
            .map_err(|e| format!("{e:?}"))
            .unwrap();
        let res = match res.get("sessionKey") {
            Some(val) => val,
            None => return Err("Couldn't get sessionKey".to_string()),
        };
        let res = match res.get("$value") {
            Some(val) => val.as_str().unwrap().to_string(),
            None => return Err("Couldn't get sessionKey.$value from response".to_string()),
        };
        self.auth_session_mode = AuthenticatedSessionMode::Token { value: res };
        Ok(())
    }

    /// Get the authenticated session owner username.
    /// <https://docs.splunk.com/Documentation/Splunk/9.0.4/RESTREF/RESTaccess#authentication.2Fcurrent-context>
    /// Currently returns just the raw XML result as a string
    #[cfg(feature = "xml_raw")]
    pub async fn get_current_context(&mut self) -> Result<String, String> {
        let endpoint = "/services/authentication/current-context";

        let res = self.do_get(endpoint).await?;
        let res = res.text().await.map_err(|e| format!("{e:?}"))?;
        // serde_xml_rs::from_str(&res).map_err(|e| format!("{e:?}"))
        Ok(res)
    }

    /// Get the authenticated session owner username.
    /// <https://docs.splunk.com/Documentation/Splunk/9.0.4/RESTREF/RESTaccess#authorization.2Fcapabilities>
    ///
    /// Currently returns just the raw XML result as a string
    #[cfg(feature = "xml_raw")]
    pub async fn get_capabilities(&mut self) -> Result<String, String> {
        let endpoint = "/services/authorization/capabilities";

        let res = self.do_get(endpoint).await?;
        let res = res.text().await.map_err(|e| format!("{e:?}"))?;
        res
    }

    /// do an export-search - TODO
    pub async fn export() -> Result<(), String> {
        unimplemented!();
    }
}
