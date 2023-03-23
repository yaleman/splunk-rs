//! HTTP Event Collector related functionality
//!
//! <https://docs.splunk.com/Documentation/Splunk/9.0.4/Data/HECExamples>
//!


/// HEC Client
#[derive(Debug)]
pub struct HecClient {
    server: String,
    port: u16,
    use_tls: bool,
    validate_ssl: bool,
}

impl Default for HecClient {
    fn default() -> Self {
        Self {
            server: "localhost".to_string(),
            port: 8089,
            use_tls: true,
            validate_ssl: true,
        }
    }
}

lazy_static!{
    static ref HEC_HEALTH_EXPECTED_RESPONSE: serde_json::Value = serde_json::json!("{\"text\":\"HEC is healthy\",\"code\":17}");
}


impl HecClient {

    /// build a url based on the server/endpoint
    /// ```
    /// use crate::hec::HecClient;
    ///
    ///
    /// ```
    fn get_url(&self, endpoint: String) -> Result<Url, String> {
        let mut url = String::new();

        url.push_str(match self.use_tls {
            true => "https",
            false => "http"
        });

        reqwest::Url::new(url).unwrap().map_err(|err| format!("{err:?}"))
    }

    async fn do_get_query(&self, endpoint: &'static str, ) -> Result<(), String> {
        let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
        unimplemented!();
    }


    pub async fn get_health(&self) -> Result<(), String> {
        let endpoint = "/services/collector/health";
        self.do_get_query(endpoint).await
    }

    /// The separate HEC health endpoint for ACK-related/enabled hosts
    pub async fn get_health_ack(&self) -> Result<(), String> {

        let endpoint = "/services/collector/health?ack=true";
        self.do_get_query(endpoint).await
    }

}

