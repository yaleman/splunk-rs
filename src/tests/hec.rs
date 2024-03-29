use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::errors::SplunkError;

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_hec_endpoint_health() -> Result<(), SplunkError> {
    use crate::hec::HecClient;
    use crate::{ServerConfig, ServerConfigType};

    let client = HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);
    let result = client.get_health().await?;

    eprintln!("result: {:?}", result);
    Ok(())
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn test_hec_endpoint_health_ack() -> Result<(), SplunkError> {
    use crate::hec::HecClient;
    use crate::{ServerConfig, ServerConfigType};

    let client = HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    let result = client.get_health_ack().await?;

    eprintln!("result: {:?}", result);
    Ok(())
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn send_test_data() -> Result<(), SplunkError> {
    use crate::hec::HecClient;

    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{ServerConfig, ServerConfigType};

    let client = HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    let now = SystemTime::now();
    let unix_time = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

    let test_event = json!({
        "test" :1, "_time" : unix_time, "message" : "Hello from splunk-rs testing",
    });

    client
        .send_event(test_event)
        .await
        .map_err(|e| SplunkError::Generic(e.to_string()))
}

#[derive(Debug, serde::Serialize)]
struct TestEvent {
    test_name: String,
    #[serde(alias = "_time")]
    time: u64,
    message: String,
}

impl TestEvent {
    #[allow(dead_code)]
    fn new(test_name: impl ToString, message: impl ToString) -> Self {
        let now = SystemTime::now();
        Self {
            test_name: test_name.to_string(),
            time: now.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            message: message.to_string(),
        }
    }
}

impl From<TestEvent> for Value {
    fn from(value: TestEvent) -> Self {
        json!(value)
    }
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn send_queued_multi_overized_batch() -> Result<(), SplunkError> {
    use crate::hec::HecClient;

    use crate::{ServerConfig, ServerConfigType};

    let mut client =
        HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    for i in 0..3 {
        let event = TestEvent::new("send_queued_multi", format!("Event {:?}", i));
        client.enqueue(event).await;
    }

    client
        .flush(Some(20))
        .await
        .map_err(|err| SplunkError::Generic(err.to_string()))?;
    Ok(())
}

// This'll turn up in the logs when you search for *monkeymonkeymonkey* sourcetype="*:access" */services/collector*
#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn send_with_custom_useragent() -> Result<(), SplunkError> {
    use crate::hec::HecClient;
    use crate::{ServerConfig, ServerConfigType};

    let mut client =
        HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    client.useragent("splunk-rs-monkeymonkeymonkey");

    for i in 0..3 {
        let event = TestEvent::new("send_with_custom_useragent", format!("Event {:?}", i));
        client.enqueue(event).await;
    }
    client
        .flush(Some(20))
        .await
        .map_err(|err| SplunkError::Generic(err.to_string()))?;
    Ok(())
}
