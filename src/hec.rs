//! HTTP Event Collector related functionality
//!
//! Based on <https://docs.splunk.com/Documentation/Splunk/9.0.4/Data/HECExamples>
//!

use log::error;
use reqwest::{header::HeaderMap, redirect::Policy, Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::search::AuthenticationMethod;
use crate::ServerConfig;

/// HEC Client
#[derive(Debug)]
pub struct HecClient {
    pub serverconfig: ServerConfig,
    pub index: Option<String>,
    pub sourcetype: Option<String>,
    pub source: Option<String>,
}

impl Default for HecClient {
    fn default() -> Self {
        Self {
            serverconfig: ServerConfig {
                hostname: "localhost".to_string(),
                port: 8088,
                verify_tls: true,
                validate_ssl: true,
                auth_method: crate::search::AuthenticationMethod::Unknown,
                ..Default::default()
            },
            index: None,
            sourcetype: None,
            source: None,
        }
    }
}

lazy_static! {
    static ref HEC_HEALTH_EXPECTED_RESPONSE: serde_json::Value =
        serde_json::json!("{\"text\":\"HEC is healthy\",\"code\":17}");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HecHealthResult {
    text: String,
    code: u32,
}

impl HecClient {
    pub fn new(token: String, hostname: String) -> Self {
        let serverconfig = ServerConfig::new(hostname).with_token(token);
        Self {
            serverconfig,
            ..Default::default()
        }
    }

    async fn do_healthcheck(&self, endpoint: &str) -> Result<HecHealthResult, String> {
        let res = self
            .serverconfig
            .do_get(endpoint, HeaderMap::new())
            .await
            .unwrap()
            .json::<HecHealthResult>()
            .await;

        res.map_err(|e| format!("{e:?}"))
    }

    pub async fn get_health(&self) -> Result<HecHealthResult, String> {
        self.do_healthcheck("/services/collector/health").await
    }

    /// The separate HEC health endpoint for ACK-related/enabled hosts
    pub async fn get_health_ack(&self) -> Result<HecHealthResult, String> {
        self.do_healthcheck("/services/collector/health?ack=true")
            .await
    }

    pub fn with_index(mut self, index: impl ToString) -> Self {
        self.index = Some(index.to_string());
        self
    }

    pub fn with_sourcetype(mut self, sourcetype: impl ToString) -> Self {
        self.sourcetype = Some(sourcetype.to_string());
        self
    }

    pub fn with_source(mut self, source: impl ToString) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// send data to the HEC endpoint
    pub async fn send_to_splunk(&self, event: Value) -> Result<(), Error> {
        // Create a reqwest Client to send the HTTP request
        let mut client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .redirect(Policy::none());

        if self.serverconfig.verify_tls {
            client = client.danger_accept_invalid_certs(true);
        }

        let client = client.build()?;

        // Create a map of headers to send with the request
        let mut headers = HeaderMap::new();
        let token = match self.serverconfig.auth_method.clone() {
            AuthenticationMethod::Token { token } => token,
            AuthenticationMethod::Basic {
                username: _,
                password,
            } => password,
            AuthenticationMethod::Unknown => {
                error!("Token is not set for HEC Event!");
                "".to_string()
            }
        };
        headers.insert(
            "Authorization",
            format!("Splunk {}", token).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        // Add index, sourcetype, and source fields to the payload if they are set
        let mut payload = json!({ "event": event });
        if let Some(index) = &self.index {
            payload["index"] = json!(index);
        }
        if let Some(sourcetype) = &self.sourcetype {
            payload["sourcetype"] = json!(sourcetype);
        }
        if let Some(source) = &self.source {
            payload["source"] = json!(source);
        }

        // Send the POST request with the payload and headers to the Splunk HEC endpoint
        let url = format!(
            "https://{}:{}/services/collector",
            self.serverconfig.hostname, self.serverconfig.port
        );
        let request_builder = client
            .post(&url)
            .headers(headers)
            .body(serde_json::to_string(&payload).unwrap());

        let result = request_builder.send().await?;

        result.error_for_status().unwrap();

        Ok(())
    }
}
