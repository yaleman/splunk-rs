//! Client for splunk enterprise/cloud API things, login, search, manipulate config items etc.
//!

use crate::errors::SplunkError;
use crate::ServerConfig;
use reqwest::header::HeaderMap;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    Token(
        /// the token
        String,
    ),
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
    pub fn with_config(self, serverconfig: ServerConfig) -> Result<Self, SplunkError> {
        let client = match serverconfig.verify_tls {
            true => Client::new(),
            false => Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?,
        };

        Ok(Self {
            serverconfig,
            client,
            ..self
        })
    }

    /// Set the authentication session mode
    pub fn with_auth_session_mode(self, mode: AuthenticatedSessionMode) -> Self {
        Self {
            auth_session_mode: mode,
            ..self
        }
    }

    /// Make a POST request
    pub async fn do_post(
        &mut self,
        endpoint: &str,
        payload: HashMap<impl Serialize, String>,
    ) -> Result<Response, SplunkError> {
        let req = self
            .client
            .post(self.serverconfig.get_url(endpoint)?)
            .form(&payload);

        let req = match &self.serverconfig.auth_method {
            AuthenticationMethod::Basic { username, password } => {
                req.basic_auth(username, Some(password))
            }
            AuthenticationMethod::Token { token } => {
                req.header("Authorization", format!("Splunk {}", token))
            }
            AuthenticationMethod::Unknown => return Err(SplunkError::NotAuthenticated),
            // TODO: handle cookie auth for posts?
            AuthenticationMethod::Cookie { cookie: _ } => req,
        };
        req.send()
            .await
            .map(|val| val.error_for_status().map_err(SplunkError::ReqwestError))?
    }

    /// Make a GET request, tries to pass the authentication automagically
    pub async fn do_get(&mut self, endpoint: &str) -> Result<Response, SplunkError> {
        let request = self.client.get(self.serverconfig.get_url(endpoint)?);

        let request = match &self.auth_session_mode {
            AuthenticatedSessionMode::Token(value) => {
                let mut headers = HeaderMap::new();
                headers.insert("Authorization", format!("Splunk {}", value).parse()?);
                request.headers(headers)
            }
            AuthenticatedSessionMode::Cookie { value: _ } => request,
            AuthenticatedSessionMode::Unset => return Err(SplunkError::NotAuthenticated),
        };

        // eprintln!("{:#?}", request);
        request
            .send()
            .await
            .map_err(|e| SplunkError::Generic(format!("{e:?}")))
    }

    /// Login and establish the session
    pub async fn login(&mut self) -> Result<(), SplunkError> {
        let endpoint = "/services/auth/login";

        let mut payload: HashMap<String, String> = HashMap::new();

        match &self.serverconfig.auth_method {
            AuthenticationMethod::Basic { username, password } => {
                // request.basic_auth(username, Some(password)),
                payload.insert("username".to_string(), username.to_owned());
                payload.insert("password".to_string(), password.to_owned());
            }
            // AuthenticationMethod::Token { token } => todo!(),
            AuthenticationMethod::Unknown => return Err(SplunkError::NoAuthMethodSelected),
            #[allow(clippy::todo)]
            _ => todo!("Token mode isn't supported!"),
        };

        let request = self.do_post(endpoint, payload).await?;

        #[cfg(test)]
        eprintln!("Headers: {:#?}", request.headers());
        let body = request.text().await?;
        #[cfg(test)]
        eprintln!("Body: {}", body);
        let res: SessionKey = serde_xml_rs::from_str(&body)?;

        #[derive(Deserialize)]
        struct SessionKey {
            #[serde(rename = "sessionKey")]
            session_key: Option<String>,
        }
        let res = match res.session_key {
            Some(val) => val,
            None => return Err(SplunkError::Generic("Couldn't get sessionKey".to_string())),
        };
        eprintln!("Body parsing OK");

        self.auth_session_mode = AuthenticatedSessionMode::Token(res);
        Ok(())
    }

    /// Get the authenticated session owner username.
    /// <https://docs.splunk.com/Documentation/Splunk/9.0.4/RESTREF/RESTaccess#authentication.2Fcurrent-context>
    /// Currently returns just the raw XML result as a string
    pub async fn get_current_context(&mut self) -> Result<String, SplunkError> {
        let endpoint = "/services/authentication/current-context";

        let res = self.do_get(endpoint).await?;
        let res = res.text().await.map_err(|e| format!("{e:?}"))?;
        Ok(res)
    }

    /// Get the authenticated session owner username.
    /// <https://docs.splunk.com/Documentation/Splunk/9.0.4/RESTREF/RESTaccess#authorization.2Fcapabilities>
    ///
    /// Currently returns just the raw XML result as a string
    pub async fn get_capabilities(&mut self) -> Result<String, SplunkError> {
        let endpoint = "/services/authorization/capabilities";

        let res = self.do_get(endpoint).await?;
        let res = res
            .text()
            .await
            .map_err(|e| SplunkError::Generic(format!("{e:?}")))?;
        Ok(res)
    }

    /// Get the saved searches from an instance
    ///
    /// This returns a [serde_json::Value] because it's a big complex mess of JSON with variable fields
    ///
    /// <https://docs.splunk.com/Documentation/Splunk/latest/RESTREF/RESTsearch#saved.2Fsearches>
    pub async fn get_saved_searches(
        &mut self,
        earliest_time: Option<&str>,
        latest_time: Option<&str>,
        // Indicates whether to list default actions.
        list_default_action_args: Option<bool>,
        add_orphan_field: Option<bool>,
        offset: Option<u32>,
    ) -> Result<ApiResponse, SplunkError> {
        let mut endpoint = "/services/saved/searches".to_string();

        let mut params = HashMap::new();

        params.insert("output_mode", "json".to_string());

        if let Some(offset) = offset {
            params.insert("offset", offset.to_string());
        }

        if let Some(earliest_time) = earliest_time {
            params.insert("earliest_time", earliest_time.to_string());
        }
        if let Some(latest_time) = latest_time {
            params.insert("latest_time", latest_time.to_string());
        }
        if let Some(list_default_action_args) = list_default_action_args {
            params.insert(
                "list_default_action_args",
                list_default_action_args.to_string(),
            );
        }
        if let Some(add_orphan_field) = add_orphan_field {
            params.insert("add_orphan_field", add_orphan_field.to_string());
        }
        // TODO: this is janky
        add_query_params_to_endpoint(&mut endpoint, &params);

        let res = self.do_get(&endpoint).await?;
        // do the query
        let res_content = res.text().await.map_err(|err| {
            SplunkError::Generic(format!(
                "Couldn't get response content from get_saved_searches: {:?}",
                err,
            ))
        })?;

        let parsed_response: ApiResponse = serde_json::from_str(&res_content).map_err(|err| {
            SplunkError::Generic(format!(
                "Couldn't parse response from get_saved_searches: {:?} - {:?}",
                err, res_content,
            ))
        })?;

        Ok(parsed_response)
    }

    /// Instead of making a single request, you can pull all the saved searches here
    pub async fn get_all_saved_searches(
        &mut self,
        earliest_time: Option<&str>,
        latest_time: Option<&str>,
        // Indicates whether to list default actions.
        list_default_action_args: Option<bool>,
        add_orphan_field: Option<bool>,
    ) -> Result<Vec<Value>, SplunkError> {
        let mut results: Vec<Value> = vec![];
        let mut offset: u32 = 0;
        loop {
            let res = self
                .get_saved_searches(
                    earliest_time,
                    latest_time,
                    list_default_action_args,
                    add_orphan_field,
                    Some(offset),
                )
                .await?;
            results.extend(res.entry.to_vec());
            if res.paging_has_more() {
                let per_page = res
                    .paging
                    .as_ref()
                    .ok_or_else(|| {
                        SplunkError::Generic(
                            "No paging info found after seeing it in initial call - this is a bug!"
                                .to_string(),
                        )
                    })?
                    .per_page;

                offset += per_page;
            } else {
                break;
            }
        }

        Ok(results)
    }
}

/// Takes a HashMap of key/value pairs to add ot the URL and adds the query values to the endpoint
pub(crate) fn add_query_params_to_endpoint(
    endpoint: &mut String,
    params: &HashMap<&str, impl ToString>,
) {
    if !params.is_empty() {
        endpoint.push('?');
        let param_strings: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::Encoded(v.to_string())))
            .collect();
        endpoint.push_str(&param_strings.join("&"));
    }
}

/// This is the "generator" element in API Responses
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponseGenerator {
    /// The Splunk build ID
    pub build: String,
    /// The Splunk version number
    pub version: String,
}

/// This is the "paging" element in API Responses
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponsePaging {
    /// Total possible responses
    pub total: u32,
    #[serde(rename = "perPage")]
    /// Number returned per page
    pub per_page: u32,
    /// Current request offset
    pub offset: u32,
}

impl ApiResponsePaging {
    /// Is there more pages?
    pub fn has_more(&self) -> bool {
        if self.offset > self.total {
            false
        } else {
            (self.total - self.per_page) > self.offset
        }
    }
}

/// Trying to capture an API response as a struct!
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    /// Links to other endpoints
    pub links: HashMap<String, String>,
    /// The endpoint which generated this response
    pub origin: String,
    /// The timestamp of the request
    pub updated: String, // TODO: parse this into an offsetdatetime
    /// Splunk version/build that generated this response
    pub generator: Option<ApiResponseGenerator>,
    /// The results
    pub entry: Vec<Value>,
    /// Information/error messages in your response
    pub messages: Vec<Value>,
    /// Details of where you are in the response set
    pub paging: Option<ApiResponsePaging>,
}

impl ApiResponse {
    /// Check that the paging indicates we have more results - if there's no paging data in the response, then you get a false regardless.
    pub fn paging_has_more(&self) -> bool {
        if let Some(paging) = &self.paging {
            paging.has_more()
        } else {
            false
        }
    }
}
