//! Client integration tests
//!

use std::collections::HashMap;

use splunk::client::{add_query_params_to_endpoint, SplunkClient};
use splunk::errors::SplunkError;
use splunk::models::responses::ApiResponsePaging;
use splunk::server_config::{ServerConfig, ServerConfigBuilder, ServerConfigType};

#[cfg_attr(feature = "test_ci", ignore)]
#[tokio::test]
async fn test_get_saved_searches() -> Result<(), SplunkError> {
    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

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
fn test_add_query_params_to_endpoint() {
    let mut endpoint = "/services/saved/searches".to_string();
    let mut params = HashMap::new();
    params.insert("earliest_time", "-1d".to_string());
    add_query_params_to_endpoint(&mut endpoint, &params);
    assert_eq!(endpoint, "/services/saved/searches?earliest_time=-1d");

    let mut endpoint_normal = "/services/saved/searches".to_string();
    let params_normal: HashMap<&str, &str> = HashMap::new();
    add_query_params_to_endpoint(&mut endpoint_normal, &params_normal);
    assert_eq!(endpoint_normal, "/services/saved/searches");
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
