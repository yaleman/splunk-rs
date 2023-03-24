
#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search_login() -> Result<(), String> {

    use crate::search::SplunkClient;
    let serverconfig = crate::tests::get_serverconfig(crate::tests::TestServerConfig::Api)?;

    let mut client = SplunkClient::new(serverconfig.username.unwrap(), serverconfig.password.unwrap());
    client.login().await.unwrap();
    Ok(())
}

#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_search() -> Result<(), String> {

    use crate::search::SplunkClient;
    use crate::search::SearchJob;
    let serverconfig = crate::tests::get_serverconfig(crate::tests::TestServerConfig::Api)?;

    let mut client = SplunkClient::new(serverconfig.username.unwrap(), serverconfig.password.unwrap());
    client.login().await.unwrap();

    let search = SearchJob::new("| makeresults 1");
    let search = search.create(&mut client).await.map_err(|e| format!("{e:?}")).unwrap();

    eprintln!("search ID: {}", search.sid);
    Ok(())
}

#[cfg(feature = "xml_raw")]
#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_get_current_context() -> Result<(), String> {

    use crate::search::SplunkClient;
    let serverconfig = crate::tests::get_serverconfig(crate::tests::TestServerConfig::Api)?;

    let mut client = SplunkClient::new(serverconfig.username.unwrap(), serverconfig.password.unwrap());
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



    let mut client = SplunkClient::new(serverconfig.username.unwrap(), serverconfig.password.unwrap());
    client.serverconfig = serverconfig;
    client.login().await?;

    eprintln!("{:#?}", client.get_capabilities().await?);

    Ok(())
}

