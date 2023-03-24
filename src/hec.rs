//! HTTP Event Collector related functionality
//!
//! <https://docs.splunk.com/Documentation/Splunk/9.0.4/Data/HECExamples>
//!

use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::ServerConfig;

/// HEC Client
#[derive(Debug)]
pub struct HecClient {
    pub serverconfig: ServerConfig,
    pub token: String,
}

impl Default for HecClient {
    fn default() -> Self {
        Self {
            serverconfig: ServerConfig {
                hostname: "localhost".to_string(),
                port: 8088,
                use_tls: true,
                validate_ssl: true,
                auth_method: crate::search::AuthenticationMethod::Unknown,
            },

            token: "".to_string(),
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
    pub fn new() -> Self {
        Self::default()
    }

    async fn do_get_query(&self, endpoint: &str) -> Result<Response, reqwest::Error> {
        reqwest::get(self.serverconfig.get_url(endpoint).unwrap()).await
        // .json::<HashMap<String, String>>()
        // .await?;
        // unimplemented!();
    }

    async fn do_healthcheck(&self, endpoint: &str) -> Result<HecHealthResult, String> {
        let res = self
            .do_get_query(endpoint)
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
}
