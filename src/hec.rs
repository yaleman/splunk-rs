//! HTTP Event Collector related functionality
//!
//! Based on <https://docs.splunk.com/Documentation/Splunk/9.0.4/Data/HECExamples>
//!

use std::cmp::min;
use std::collections::VecDeque;
use std::sync::Arc;

use log::error;
use reqwest::{header::HeaderMap, redirect::Policy, Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::RwLock;

use crate::search::AuthenticationMethod;
use crate::ServerConfig;

/// HEC Client
#[derive(Debug)]
pub struct HecClient {
    /// See [ServerConfig]
    pub serverconfig: ServerConfig,
    /// The target index - if this is None then it'll just let the server decide
    pub index: Option<String>,
    /// The target sourcetype - if this is None then it'll just let the server decide
    pub sourcetype: Option<String>,
    /// The target source - if this is None then it'll just let the server decide
    pub source: Option<String>,
    queue: Arc<RwLock<VecDeque<Box<Value>>>>,
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
            queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
}

lazy_static! {
    static ref HEC_HEALTH_EXPECTED_RESPONSE: serde_json::Value =
        serde_json::json!("{\"text\":\"HEC is healthy\",\"code\":17}");
}

#[derive(Debug, Serialize, Deserialize)]
/// Deserializer for the response from HEC Health Checks
pub struct HecHealthResult {
    text: String,
    code: u32,
}

impl HecClient {
    /// Create a new HEC client, specifying the token and hostname. Defaults to port 8088
    pub fn new(token: impl ToString, hostname: impl ToString) -> Self {
        let serverconfig = ServerConfig::new(hostname.to_string())
            .with_token(token.to_string())
            .with_port(8088);
        Self {
            serverconfig,
            ..Default::default()
        }
    }

    /// Start the HEC Client with a given server config
    pub fn with_serverconfig(serverconfig: ServerConfig) -> Self {
        Self {
            serverconfig,
            ..Default::default()
        }
    }

    async fn do_healthcheck(&self, endpoint: &str) -> Result<HecHealthResult, String> {
        let res = self
            .serverconfig
            .do_get(endpoint)
            .await
            .unwrap()
            .json::<HecHealthResult>()
            .await;

        res.map_err(|e| format!("{e:?}"))
    }

    /// Do a healthcheck and return the response
    pub async fn get_health(&self) -> Result<HecHealthResult, String> {
        self.do_healthcheck("/services/collector/health").await
    }

    /// The separate HEC health endpoint for ACK-related/enabled hosts
    pub async fn get_health_ack(&self) -> Result<HecHealthResult, String> {
        self.do_healthcheck("/services/collector/health?ack=true")
            .await
    }

    /// Set the index on the events you'll send
    pub fn with_index(mut self, index: impl ToString) -> Self {
        self.index = Some(index.to_string());
        self
    }

    /// Set the sourcetype on all events you send
    pub fn with_sourcetype(mut self, sourcetype: impl ToString) -> Self {
        self.sourcetype = Some(sourcetype.to_string());
        self
    }

    /// Set the source on all events you send
    pub fn with_source(mut self, source: impl ToString) -> Self {
        self.source = Some(source.to_string());
        self
    }

    /// send a single event to the HEC endpoint
    pub async fn send_event(&self, event: Value) -> Result<(), Error> {
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

            // TODO: does HEC handle cookie auth? I don't think so?
            AuthenticationMethod::Cookie { cookie: _ } => unimplemented!("Can't use"),
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

    /// send data to the HEC endpoint
    pub async fn send_events(&self, events: Vec<Value>) -> Result<(), Error> {
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
            // TODO: does HEC handle cookie auth? I don't think so.
            AuthenticationMethod::Cookie { cookie: _ } => todo!(),
        };
        headers.insert(
            "Authorization",
            format!("Splunk {}", token).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let payload_vec: Vec<String> = events
            .into_iter()
            .map(|event| {
                // Add index, sourcetype, and source fields to the payload if they are set - but not already in the event!
                let mut payload = json!({ "event": event });

                if let Some(index) = &self.index {
                    payload["index"] = Value::String(index.to_owned());
                }
                if let Some(sourcetype) = &self.sourcetype {
                    payload["sourcetype"] = Value::String(sourcetype.to_owned());
                }
                if let Some(source) = &self.source {
                    payload["source"] = Value::String(source.to_owned());
                }
                serde_json::to_string(&payload).unwrap()
            })
            .collect();

        let payload = payload_vec.join("\n");

        // Send the POST request with the payload and headers to the Splunk HEC endpoint
        let url = format!(
            "https://{}:{}/services/collector",
            self.serverconfig.hostname, self.serverconfig.port
        );
        let request_builder = client.post(&url).headers(headers).body(payload);

        let result = request_builder.send().await?;

        result.error_for_status().unwrap();

        Ok(())
    }

    /// add a new queue item
    pub async fn enqueue(&mut self, event: Value) {
        self.queue.write().await.push_back(Box::new(event))
    }

    /// get the current queue size
    pub async fn queue_size(&self) -> usize {
        self.queue.read().await.len()
    }

    /// Flush the queue out to HEC, defaults to batches of 1000
    pub async fn flush(&mut self, batch_size: Option<u32>) -> Result<usize, Error> {
        if self.queue.read().await.is_empty() {
            return Ok(0);
        }

        let batch_size = batch_size.unwrap_or(1000);

        let mut sent: usize = 0;
        loop {
            if self.queue.read().await.is_empty() {
                break;
            }
            let mut queue = self.queue.write().await;
            let queue_len = queue.len();
            let events = queue.drain(0..min(queue_len, batch_size as usize));
            // TODO: handle max payload size, because sometimes posting a gig of data is bad
            let payload: Vec<Value> = events.into_iter().map(|e| *e).collect();
            sent += payload.len();

            self.send_events(payload).await?;
        }

        Ok(sent)
    }
}
