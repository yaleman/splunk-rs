//! Search integration tests
//!

use splunk::client::SplunkClient;
use splunk::errors::SplunkError;
use splunk::server_config::{ServerConfig, ServerConfigType};

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search_login() -> Result<(), SplunkError> {
    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    eprintln!("{:?}", serverconfig);

    let mut client = SplunkClient::default().with_config(serverconfig)?;
    eprintln!("{:?}", client);
    client.login().await?;
    Ok(())
}

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search_execution() -> Result<(), SplunkError> {
    use splunk::search::SearchJob;

    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    eprintln!("{:?}", serverconfig);

    let mut client: SplunkClient = SplunkClient::default().with_config(serverconfig)?;
    println!("{:#?}", client.serverconfig);

    client.login().await?;

    let search_string =
        r#"| makeresults 1 | eval foo="12345,12345" | makemv foo delim="," | mvexpand foo"#;
    println!("search string: {}", search_string);
    let search = SearchJob::create(search_string);

    let search = search.create(&mut client).await?;
    search
        .map(|result| {
            let resultline: splunk::search::SearchResult = serde_json::from_str(&result)?;
            println!("{:#?}", resultline);
            Ok(())
        })
        .await?;

    Ok(())
}
