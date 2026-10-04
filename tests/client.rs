//! Client integration tests
//!

use splunk::client::SplunkClient;
use splunk::errors::SplunkError;
use splunk::models::responses::ApiResponsePaging;
use splunk::server_config::ServerConfigBuilder;

use httpmock::prelude::*;

#[tokio::test]
async fn test_get_saved_searches() -> Result<(), SplunkError> {
    // lightweight mock server.
    let server = MockServer::start();
    eprintln!("Mock server started at {}", server.url(""));

    // Create a mock on the server.
    let mock_login = server.mock(|when, then| {
        when.method(POST)
            .path("/services/auth/login")
            .form_urlencoded_tuple_matches("username", "admin")
            .form_urlencoded_tuple_matches("password", "Admin1234!");
        then.status(200)
            .header("content-type", "text/xml; charset=UTF-8")
            .body(
                r#"<response>
  <sessionKey>fakesessionkey</sessionKey>
  <messages>
    <msg code=""></msg>
  </messages>
</response>"#,
            );
    });
    eprintln!("Created mock login response");

    let serverconfig = ServerConfigBuilder::default()
        .use_tls(false)
        .with_hostname(server.host())
        .with_port(server.port())
        .with_username_password("admin", "Admin1234!")
        .with_connection_timeout(5)
        .build()?;

    let mut client = SplunkClient::new()?.with_config(serverconfig)?;

    client.login().await?;

    assert_eq!(mock_login.calls(), 1);
    mock_login.assert();

    eprintln!("Login worked, resetting...");
    server.reset();
    let mock_body = std::fs::read_to_string(
        env!("CARGO_MANIFEST_DIR").to_string() + "/tests/saved-searches-mock-response.json",
    )
    .expect("Failed to read mock response file");
    assert!(!mock_body.is_empty(), "Mock body should not be empty");
    let mock_searches = server.mock(|when, then| {
        when.method(GET).path("/services/saved/searches");
        then.status(200)
            .header("content-type", "application/json; charset=UTF-8")
            .body(&mock_body);
    });

    let earliest = Some("-1d");
    eprintln!("Fetching saved searches with earliest = {:?}", earliest);
    let saved_searches = client
        .get_all_saved_searches(earliest, None, None, None)
        .await?;

    assert_eq!(mock_searches.calls(), 1);
    mock_searches.assert();

    println!("{}", serde_json::to_string_pretty(&saved_searches)?);
    assert_eq!(
        saved_searches.len(),
        1,
        "Got {} saved searches, should have gotten 1",
        saved_searches.len()
    );

    Ok(())
}

#[tokio::test]
async fn test_login() -> Result<(), SplunkError> {
    // lightweight mock server.
    let server = MockServer::start();
    eprintln!("Mock server started at {}", server.url(""));

    let mock_login_pw_wrong = server.mock(|when, then| {
        when.method(POST)
            .path("/services/auth/login")
            .form_urlencoded_tuple_matches("username", "admin")
            .form_urlencoded_tuple_matches("password", "wrong");
        then.status(401)
            .header("content-type", "text/xml; charset=UTF-8")
            .body(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<response>
  <messages>
    <msg type="WARN" code="incorrect_username_or_password">Login failed</msg>
  </messages>
</response>"#,
            );
    });
    let mock_login_auth_both_wrong = server.mock(|when, then| {
        when.method(POST)
            .path("/services/auth/login")
            .form_urlencoded_tuple_not("username", "admin")
            .form_urlencoded_tuple_not("password", "Admin1234!");
        then.status(401)
            .header("content-type", "text/xml; charset=UTF-8")
            .body(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<response>
  <messages>
    <msg type="WARN" code="incorrect_username_or_password">Login failed</msg>
  </messages>
</response>"#,
            );
    });
    // Create a mock on the server.
    let mock_login_auth_missing = server.mock(|when, then| {
        when.method(POST)
            .path("/services/auth/login")
            .form_urlencoded_tuple_missing("username")
            .form_urlencoded_tuple_missing("password");
        then.status(400)
            .header("content-type", "text/xml; charset=UTF-8")
            .body(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<response>
  <messages>
    <msg type="WARN">Login failed</msg>
  </messages>
</response>"#,
            );
    });

    let mock_login_ok = server.mock(|when, then| {
        when.method(POST)
            .path("/services/auth/login")
            .form_urlencoded_tuple_matches("username", "admin")
            .form_urlencoded_tuple_matches("password", "Admin1234!");
        then.status(200)
            .header("content-type", "text/xml; charset=UTF-8")
            .body(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<response>
  <sessionKey>fakesessionkey</sessionKey>
  <messages>
    <msg code=""></msg>
  </messages>
</response>"#,
            );
    });
    eprintln!("Created mock login response");

    for (username, password, should_work) in [
        ("admin", "Admin1234!", true),
        ("wrong", "wrong", false),
        ("admin", "wrong", false),
    ]
    .into_iter()
    {
        let serverconfig = ServerConfigBuilder::default()
            .use_tls(false)
            .with_hostname(server.host())
            .with_port(server.port())
            .with_username_password(username, password)
            .build()?;

        let mut client = SplunkClient::new()?.with_config(serverconfig)?;

        if should_work {
            client.login().await.unwrap_or_else(|_| {
                panic!(
                    "Failed with username: {} and password: {}",
                    username, password
                )
            });
        } else {
            assert!(
                client.login().await.is_err(),
                "Login should have failed for username: {} and password: {}",
                username,
                password
            );
        }
    }
    assert_eq!(mock_login_ok.calls(), 1);
    mock_login_ok.assert();
    assert_eq!(
        mock_login_auth_both_wrong.calls(),
        1,
        "Should have been called once for entirely wrong credentials"
    );

    assert_eq!(
        mock_login_pw_wrong.calls(),
        1,
        "Should have been called once for wrong password"
    );
    mock_login_pw_wrong.assert();

    assert_eq!(
        mock_login_auth_missing.calls(),
        0,
        "Login should always include username and password form fields"
    );
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
