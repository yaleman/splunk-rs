#[tokio::test]
#[cfg_attr(feature = "test_ci", ignore)]
async fn test_hec_endpoint_health() -> Result<(), String> {
    use crate::hec::HecClient;

    let client = HecClient {
        serverconfig: crate::tests::get_serverconfig(crate::tests::TestServerConfig::Hec)?,
        token: "".to_string(),
    };

    let result = client.get_health().await?;

    eprintln!("result: {:?}", result);
    Ok(())
}

#[cfg_attr(feature="test_ci", ignore)]
#[tokio::test]
async fn test_hec_endpoint_health_ack() -> Result<(), String> {


    use crate::hec::HecClient;

    let client = HecClient {
        serverconfig: crate::tests::get_serverconfig(crate::tests::TestServerConfig::Hec)?,
        token: "".to_string(),
    };

    let result = client.get_health_ack().await?;

    eprintln!("result: {:?}", result);
    Ok(())
}
