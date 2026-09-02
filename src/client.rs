//! Client for splunk enterprise/cloud API things, login, search, manipulate config items etc.
//!

use crate::errors::SplunkError;
use crate::models::requests::BasicAuth;
use crate::models::responses::ApiResponse;
use crate::models::{AuthenticatedSessionMode, AuthenticationMethod};
use crate::server_config::ServerConfig;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE};
use reqwest::{Client, Response, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Build a [`Client`] with the auth session's credentials baked in as default headers,
/// so callers don't need to attach `Authorization`/`Cookie` headers on every request.
fn build_client(
    verify_tls: bool,
    auth_session_mode: &AuthenticatedSessionMode,
) -> Result<Client, SplunkError> {
    let mut headers = HeaderMap::new();

    match auth_session_mode {
        AuthenticatedSessionMode::Token(value) => {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Splunk {}", value))?,
            );
        }
        AuthenticatedSessionMode::Cookie { value } => {
            let cookie_str = value
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("; ");
            headers.insert(COOKIE, HeaderValue::from_str(&cookie_str)?);
        }
        AuthenticatedSessionMode::Unset => {}
    }

    let mut builder = Client::builder().default_headers(headers);
    if !verify_tls {
        builder = builder.danger_accept_invalid_certs(true);
    }

    builder.build().map_err(SplunkError::ReqwestError)
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
        let client = build_client(serverconfig.verify_tls, &self.auth_session_mode)?;

        Ok(Self {
            serverconfig,
            client,
            ..self
        })
    }

    /// Set the authentication session mode, rebuilding the client so the credentials are
    /// baked in as default headers for every subsequent request.
    pub fn with_auth_session_mode(
        mut self,
        mode: AuthenticatedSessionMode,
    ) -> Result<Self, SplunkError> {
        self.set_auth_session_mode(mode)?;
        Ok(self)
    }

    /// Make a POST request against a fully-resolved [`Url`] (see [`ServerConfig::get_url`] to
    /// build one from an endpoint, and [`Url::query_pairs_mut`] to add query parameters).
    /// Authentication is already baked into the client's default headers by
    /// [`SplunkClient::login`], so no per-request setup is needed here.
    pub async fn do_post(
        &mut self,
        url: Url,
        payload: impl Serialize,
    ) -> Result<Response, SplunkError> {
        if matches!(self.auth_session_mode, AuthenticatedSessionMode::Unset) {
            return Err(SplunkError::NotAuthenticated);
        }

        self.client
            .post(url)
            .form(&payload)
            .send()
            .await
            .map(|val| val.error_for_status().map_err(SplunkError::ReqwestError))?
    }

    /// Make a GET request against a fully-resolved [`Url`] (see [`ServerConfig::get_url`] to
    /// build one from an endpoint, and [`Url::query_pairs_mut`] to add query parameters).
    /// Authentication is already baked into the client's default headers by
    /// [`SplunkClient::login`], so no per-request setup is needed here.
    pub async fn do_get(&mut self, url: Url) -> Result<Response, SplunkError> {
        if matches!(self.auth_session_mode, AuthenticatedSessionMode::Unset) {
            return Err(SplunkError::NotAuthenticated);
        }

        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| SplunkError::Generic(format!("{e:?}")))
    }

    /// Login and establish the session, dispatching to the flow matching the configured
    /// [`AuthenticationMethod`].
    pub async fn login(&mut self) -> Result<(), SplunkError> {
        match self.serverconfig.auth_method.clone() {
            AuthenticationMethod::Basic { username, password } => {
                self.login_with_basic_auth(&username, &password).await
            }
            AuthenticationMethod::Token { token } => self.login_with_token(&token),
            AuthenticationMethod::Unknown => Err(SplunkError::NoAuthMethodSelected),
            AuthenticationMethod::Cookie { .. } => Err(SplunkError::InvalidAuthmethod(
                "Cookie auth method can't be used to log in",
            )),
        }
    }

    /// Exchange a username/password for a session token via `/services/auth/login`.
    pub async fn login_with_basic_auth(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(), SplunkError> {
        let url = self.serverconfig.get_url("/services/auth/login")?;
        let payload = BasicAuth { username, password };

        let request = self
            .client
            .post(url)
            .form(&payload)
            .send()
            .await?
            .error_for_status()?;

        let body = request.text().await?;
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

        self.set_auth_session_mode(AuthenticatedSessionMode::Token(res))
    }

    /// "Login" using a pre-issued token. Splunk token auth doesn't require an
    /// `/services/auth/login` exchange - the token is used directly as the session credential.
    pub fn login_with_token(&mut self, token: &str) -> Result<(), SplunkError> {
        self.set_auth_session_mode(AuthenticatedSessionMode::Token(token.to_string()))
    }

    /// Set the authentication session mode and rebuild the underlying HTTP client so the
    /// bearer token / cookies are baked in as default headers for every subsequent request.
    fn set_auth_session_mode(&mut self, mode: AuthenticatedSessionMode) -> Result<(), SplunkError> {
        self.client = build_client(self.serverconfig.verify_tls, &mode)?;
        self.auth_session_mode = mode;
        Ok(())
    }

    /// Get the authenticated session owner username.
    /// <https://docs.splunk.com/Documentation/Splunk/9.0.4/RESTREF/RESTaccess#authentication.2Fcurrent-context>
    /// Currently returns just the raw XML result as a string
    pub async fn get_current_context(&mut self) -> Result<String, SplunkError> {
        let url = self
            .serverconfig
            .get_url("/services/authentication/current-context")?;

        let res = self.do_get(url).await?;
        let res = res.text().await.map_err(|e| format!("{e:?}"))?;
        Ok(res)
    }

    /// Get the authenticated session owner username.
    /// <https://docs.splunk.com/Documentation/Splunk/9.0.4/RESTREF/RESTaccess#authorization.2Fcapabilities>
    ///
    /// Currently returns just the raw XML result as a string
    pub async fn get_capabilities(&mut self) -> Result<String, SplunkError> {
        let url = self
            .serverconfig
            .get_url("/services/authorization/capabilities")?;

        let res = self.do_get(url).await?;
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
        let mut url = self.serverconfig.get_url("/services/saved/searches")?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("output_mode", "json");

            if let Some(offset) = offset {
                query.append_pair("offset", &offset.to_string());
            }
            if let Some(earliest_time) = earliest_time {
                query.append_pair("earliest_time", earliest_time);
            }
            if let Some(latest_time) = latest_time {
                query.append_pair("latest_time", latest_time);
            }
            if let Some(list_default_action_args) = list_default_action_args {
                query.append_pair(
                    "list_default_action_args",
                    &list_default_action_args.to_string(),
                );
            }
            if let Some(add_orphan_field) = add_orphan_field {
                query.append_pair("add_orphan_field", &add_orphan_field.to_string());
            }
        }

        let res = self.do_get(url).await?;
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
