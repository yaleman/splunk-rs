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
