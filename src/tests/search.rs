#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search_login() -> Result<(), String> {
    use crate::search::SplunkClient;
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
async fn test_search_execution() -> Result<(), String> {
    use crate::search::SearchJob;
    use crate::search::SplunkClient;
    use crate::{ServerConfig, ServerConfigType};
    use futures::stream::TryStreamExt; // for map_err
    use tokio::io::AsyncBufReadExt;
    use tokio_util::io::StreamReader;

    let serverconfig = ServerConfig::try_from_env(ServerConfigType::Api)?;

    eprintln!("{:?}", serverconfig);

    let mut client = SplunkClient::default().with_config(serverconfig);
    println!("{:#?}", client.serverconfig);

    client.login().await?;

    let search_string =
        r#"| makeresults 1 | eval foo="12345,12345" | makemv foo delim="," | mvexpand foo"#;
    println!("search string: {}", search_string);
    let search = SearchJob::create(search_string);

    let search = search
        .create(&mut client)
        .await
        .map_err(|e| format!("{e:?}"))
        .unwrap();
    let stream = search.creation_response.bytes_stream();

    fn convert_err(_err: reqwest::Error) -> std::io::Error {
        todo!()
    }

    let mut lines = get_lines! {stream};

    while let Some(line) = lines.next_line().await.unwrap() {
        // println!("{line:?}");
        let resultline: crate::search::SearchResult = serde_json::from_str(&line).unwrap();
        println!("{:#?}", resultline);
    }
    println!("Done printing lines...");

    Ok(())
}

#[cfg(feature = "xml_raw")]
#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_get_current_context() -> Result<(), String> {
    use crate::search::SplunkClient;
    let serverconfig = crate::tests::get_serverconfig(crate::tests::TestServerConfig::Api)?;

    let mut client = SplunkClient::new().with_config(serverconfig);
    client.serverconfig = serverconfig;
    client.login().await?;

    eprintln!("{:#?}", client.get_current_context().await?);

    Ok(())
}

#[cfg(feature = "xml_raw")]
#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_get_capabilities() -> Result<(), String> {
    use crate::search::SplunkClient;
    let serverconfig = crate::tests::get_serverconfig(crate::tests::TestServerConfig::Api)?;

    let mut client = SplunkClient::new().with_config(serverconfig);
    client.serverconfig = serverconfig;
    client.login().await?;

    eprintln!("{:#?}", client.get_capabilities().await?);

    Ok(())
}
