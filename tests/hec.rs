//! HEC integration tests
//!

use std::time::{SystemTime, UNIX_EPOCH};

use httpmock::MockServer;
use serde_json::json;

use splunk::errors::SplunkError;
use splunk::hec::HecClient;
use splunk::server_config::{ServerConfig, ServerConfigType};

#[tokio::test]
async fn test_hec_endpoint_health() {
    let server = MockServer::start();
    let health_mock = server.mock(|when, then| {
        when.method("GET").path("/services/collector/health");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .json_body(json!({"text":"HEC is healthy","code":17}));
    });

    let server_config = ServerConfig::builder(server.host())
        .with_port(server.port())
        .use_tls(false)
        .with_token("no-token-needed".to_string())
        .build()
        .expect("Failed to build server config");
    let client = HecClient::with_serverconfig(server_config);
    let result = client.get_health().await.expect("Failed to get health");

    health_mock.assert();

    eprintln!("result: {:?}", result);
}

#[tokio::test]
async fn test_hec_endpoint_health_ack() {
    let server = MockServer::start();
    let health_mock = server.mock(|when, then| {
        when.method("GET")
            .path("/services/collector/health")
            .query_param_matches("ack", "true");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .json_body(json!({"text":"HEC is healthy","code":17}));
    });
    let server_config = ServerConfig::builder(server.host())
        .with_port(server.port())
        .use_tls(false)
        .with_token("no-token-needed".to_string())
        .build()
        .expect("Failed to build server config");
    let client = HecClient::with_serverconfig(server_config);

    let result = client
        .get_health_ack()
        .await
        .expect("Failed to get health ack");
    health_mock.assert();

    eprintln!("result: {:?}", result);
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn send_test_data() -> Result<(), SplunkError> {
    let client = HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    let now = SystemTime::now();
    let unix_time = now.duration_since(UNIX_EPOCH)?.as_secs();

    let test_event = json!({
        "test" :1, "_time" : unix_time, "message" : "Hello from splunk-rs testing",
    });

    client.send_event(test_event).await
}

#[derive(Debug, serde::Serialize)]
struct TestEvent {
    test_name: String,
    #[serde(alias = "_time")]
    time: u64,
    message: String,
}

impl TestEvent {
    fn new(test_name: &str, message: &str) -> Self {
        let now = SystemTime::now();
        Self {
            test_name: test_name.to_string(),
            time: now
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
            message: message.to_string(),
        }
    }
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn send_queued_multi_overized_batch() -> Result<(), SplunkError> {
    let mut client =
        HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    for i in 0..3 {
        let event = TestEvent::new("send_queued_multi", &format!("Event {:?}", i));
        client.enqueue(event).await;
    }

    client.flush(Some(20)).await?;
    Ok(())
}

// This'll turn up in the logs when you search for *monkeymonkeymonkey* sourcetype="*:access" */services/collector*
#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn send_with_custom_useragent() -> Result<(), SplunkError> {
    let mut client =
        HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    client.useragent("splunk-rs-monkeymonkeymonkey");

    for i in 0..3 {
        let event = TestEvent::new("send_with_custom_useragent", &format!("Event {:?}", i));
        client.enqueue(event).await;
    }
    client.flush(Some(20)).await?;

    Ok(())
}
