#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_hec_endpoint_health() -> Result<(), String> {
    use crate::hec::HecClient;

    let client = HecClient {
        serverconfig: crate::tests::get_serverconfig(crate::tests::TestServerConfig::Hec)?,
        token: "".to_string(),
        ..Default::default()
    };

    let result = client.get_health().await?;

    eprintln!("result: {:?}", result);
    Ok(())
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn test_hec_endpoint_health_ack() -> Result<(), String> {
    use crate::hec::HecClient;

    let client = HecClient {
        serverconfig: crate::tests::get_serverconfig(crate::tests::TestServerConfig::Hec)?,
        token: "".to_string(),
        ..Default::default()
    };

    let result = client.get_health_ack().await?;

    eprintln!("result: {:?}", result);
    Ok(())
}

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn send_test_data() {
    use crate::hec::HecClient;
    use serde_json::json;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut client = HecClient::new(
        env::var("SPLUNK_HEC_TOKEN").unwrap(),
        env::var("SPLUNK_HEC_HOSTNAME").unwrap(),
    )
    .with_index("test".to_string())
    .with_source("splunk-rs test");

    let splunk_port = env::var("SPLUNK_HEC_PORT").unwrap_or(String::from("8088"));

    // Parse the port string into a u16
    let port: u16 = splunk_port.parse().unwrap();
    client.serverconfig.port = port;

    let now = SystemTime::now();
    let unix_time = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

    client
        .send_to_splunk(json!(format!("{{\"test\" : 1, \"_time\" : {unix_time} }}")))
        .await
        .unwrap();
}
