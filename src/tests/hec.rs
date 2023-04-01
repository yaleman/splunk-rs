use std::time::{UNIX_EPOCH, SystemTime};

use serde_json::{Value, json};

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_hec_endpoint_health() -> Result<(), String> {
    use crate::hec::HecClient;
    use crate::{ServerConfig, ServerConfigType};

    let client = HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);
    let result = client.get_health().await?;

    eprintln!("result: {:?}", result);
    Ok(())
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn test_hec_endpoint_health_ack() -> Result<(), String> {
    use crate::hec::HecClient;
    use crate::{ServerConfig, ServerConfigType};

    let client = HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    let result = client.get_health_ack().await?;

    eprintln!("result: {:?}", result);
    Ok(())
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn send_test_data() -> Result<(), String> {
    use crate::hec::HecClient;

    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{ServerConfig, ServerConfigType};

    let client = HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    let now = SystemTime::now();
    let unix_time = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

    client
        .send_event(json!(format!("{{\"test\" : 1, \"_time\" : {unix_time}, \"message\" : \"Hello from splunk-rs testing\" }}")))
        .await.map_err(|e| e.to_string())
}



#[derive(serde::Serialize)]
struct TestEvent {
    test_name: String,
    #[serde(alias="_time")]
    time: u64,
    message: String,
}

impl TestEvent {
    #[allow(dead_code)]
    fn new(test_name: impl ToString, message: impl ToString ) -> Self {

        let now = SystemTime::now();
        Self {
            test_name: test_name.to_string(),
            time: now.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            message: message.to_string()
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
async fn send_queued_multi_overized_batch() -> Result<(), String> {
    use crate::hec::HecClient;

    use crate::{ServerConfig, ServerConfigType};

    let mut client = HecClient::with_serverconfig(ServerConfig::try_from_env(ServerConfigType::Hec)?);

    for i in [0..3].iter() {
        let event = TestEvent::new("send_queued_multi", format!("Event {:?}", i));
        client.enqueue(event.into());
    }
    client.flush(Some(20)).await.map_err(|e| e.to_string())?;
    Ok(())
}
