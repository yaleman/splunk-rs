use crate::client::SplunkClient;
use crate::errors::SplunkError;

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search_login() -> Result<(), SplunkError> {
    use crate::{ServerConfig, ServerConfigType};

    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    eprintln!("{:?}", serverconfig);

    let mut client = SplunkClient::default().with_config(serverconfig);
    eprintln!("{:?}", client);
    client.login().await.unwrap();
    Ok(())
}

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search_execution() -> Result<(), SplunkError> {
    use crate::search::SearchJob;
    use crate::{ServerConfig, ServerConfigType};

    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    eprintln!("{:?}", serverconfig);

    let mut client = SplunkClient::default().with_config(serverconfig);
    println!("{:#?}", client.serverconfig);

    client.login().await?;

    let search_string =
        r#"| makeresults 1 | eval foo="12345,12345" | makemv foo delim="," | mvexpand foo"#;
    println!("search string: {}", search_string);
    let search = SearchJob::create(search_string);

    let search = search.create(&mut client).await?;
    search
        .map(|result| {
            let resultline: crate::search::SearchResult = serde_json::from_str(&result)?;
            println!("{:#?}", resultline);
            Ok(())
        })
        .await?;

    Ok(())
}

// #[cfg(feature = "xml_raw")]
// #[tokio::test]
// #[cfg_attr(feature = "test_ci", ignore)]
// async fn test_get_current_context() -> Result<(), SplunkError> {
//     let serverconfig = crate::tests::get_serverconfig(crate::ServerConfigType::Api)?;

//     let mut client = SplunkClient::default().with_config(serverconfig);
//     client.serverconfig = serverconfig;
//     client.login().await?;

//     eprintln!("{:#?}", client.get_current_context().await?);

//     Ok(())
// }

// #[cfg(feature = "xml_raw")]
// #[tokio::test]
// #[cfg_attr(feature = "test_ci", ignore)]
// async fn test_get_capabilities() -> Result<(), SplunkError> {
//     use crate::search::SplunkClient;
//     let serverconfig = crate::tests::get_serverconfig(crate::tests::TestServerConfig::Api)?;

//     let mut client = SplunkClient::new().with_config(serverconfig);
//     client.serverconfig = serverconfig;
//     client.login().await?;

//     eprintln!("{:#?}", client.get_capabilities().await?);

//     Ok(())
// }
