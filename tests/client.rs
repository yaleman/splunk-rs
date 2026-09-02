//! Client integration tests
//!

use std::collections::HashMap;

use splunk::client::SplunkClient;
use splunk::errors::SplunkError;
use splunk::models::responses::ApiResponsePaging;
use splunk::server_config::{ServerConfig, ServerConfigBuilder, ServerConfigType};

#[tokio::test]
async fn test_get_saved_searches() -> Result<(), SplunkError> {
    let serverconfig = ServerConfigBuilder::default()
        .use_tls(true)
        .with_hostname("localhost")
        .with_port(8089)
        .with_username_password("admin", "Admin1234!")
        .with_verify_tls(false)
        .build()?;

    let mut client = SplunkClient::default().with_config(serverconfig)?;

    client.login().await?;

    let earliest = Some("-1d");
    let saved_searches = client
        .get_all_saved_searches(earliest, None, None, None)
        .await?;

    println!("{}", serde_json::to_string_pretty(&saved_searches)?);
    println!("Got {} saved searches", saved_searches.len());

    Ok(())
}

#[tokio::test]
async fn test_login() -> Result<(), SplunkError> {
    let serverconfig = ServerConfigBuilder::default()
        .use_tls(true)
        .with_hostname("localhost")
        .with_port(8089)
        .with_username_password("admin", "Admin1234!")
        .with_verify_tls(false)
        .build()?;

    let mut client = SplunkClient::default().with_config(serverconfig)?;

    client.login().await?;

    Ok(())
}

#[test]
fn test_apiresponsepaging_has_more() {
    let testone = ApiResponsePaging {
        total: 157,
        per_page: 30,
        offset: 0,
    };
    assert!(testone.has_more());
}
