# Changelog

## 0.2.0

### Changes

- Reworked server configuration into a builder with a fixed connection URL. HEC sends now respect the configured HTTP/HTTPS scheme and port.
- Added token login and cookie session headers for REST requests. Deserializing a REST client now restores its authentication headers and HTTP settings.
- Fixed saved-search pagination when a page is larger than the total result count.
- Added configurable request deadlines: REST requests and HEC sends default to 30 seconds; search exports use a 600-second deadline and retain the 24-hour server-side job timeout. Request deadlines cover the full response body, including streaming results.
- `SplunkError` now implements `std::error::Error` and `Display`.
- Updated dependencies, including reqwest from 0.12 to 0.13, and added mocked login, saved-search and HEC health tests.

### Migrating from 0.1.3

This release changes public APIs. Update imports and construction as follows:

| 0.1.3 | 0.2.0 |
| --- | --- |
| `splunk::{ServerConfig, ServerConfigType}` | `splunk::server_config::{ServerConfig, ServerConfigType}` |
| `splunk::client::{AuthenticationMethod, AuthenticatedSessionMode}` | `splunk::models::{AuthenticationMethod, AuthenticatedSessionMode}` |
| `splunk::client::{ApiResponse, ApiResponseGenerator, ApiResponsePaging}` | `splunk::models::responses::{ApiResponse, ApiResponseGenerator, ApiResponsePaging}` |
| `ServerConfig::new(host)` followed by configuration methods | `ServerConfig::builder(host)` followed by configuration methods and `.build()?` |
| `SplunkClient::default()` | `SplunkClient::new()?` |
| `HecClient::new(token, host)` | `HecClient::new(token, host)?` |
| `.with_auth_session_mode(mode)` | `.with_auth_session_mode(mode)?` |
| `hec.timeout = seconds` | Set `.with_request_timeout(seconds)` on the server-config builder |

For example:

```rust
use splunk::client::SplunkClient;
use splunk::server_config::ServerConfig;

let config = ServerConfig::builder("splunk.example.com")
    .with_port(8089)
    .with_username_password("admin", "password")
    .with_request_timeout(60)
    .build()?;
let mut client = SplunkClient::new()?.with_config(config)?;
client.login().await?;
```

- Configure hostname, port, TLS and credentials on the builder before calling `build()`. `ServerConfig` no longer exposes mutable hostname/port fields; inspect `config.url()` or build a replacement config. `ServerConfig::try_from_env(...)` still returns a finished config; use `ServerConfigBuilder::try_from_env(...)` when you need overrides before building.
- REST `do_get` now takes a `Url`. REST `do_post` now takes `(Url, payload, Option<u16>)`, where the last argument overrides the request deadline in seconds and `None` uses the configured value. Its payload accepts any form-serializable type. Build URLs with `client.serverconfig.get_url(endpoint)?` and add query parameters with `url.query_pairs_mut()`.
- Authenticate before REST POST requests as well as GET requests: call `login().await?`, `login_with_token(token)?`, or `with_auth_session_mode(mode)?`. The `auth_session_mode` field is now private. HEC clients continue to use their configured credentials directly.
- `ServerConfig::get_url` now returns `Result<Url, SplunkError>` instead of `Result<Url, String>` and resolves endpoints using URL joining.
- Saved configuration/client JSON has changed: hostname, port and `use_tls` are replaced by `url`, and `connection_timeout` is replaced by `request_timeout`. Rebuild old saved configurations or convert those fields before deserializing.
- Allow for the new total request deadlines when exporting large searches. `SearchJobBuilder` currently uses a default 600-second network deadline with setter `request_timeout`. Direct REST POST callers can supply a longer deadline through `do_post`.
- Code sharing reqwest request/response types with this crate should use reqwest 0.13.

## 0.1.1-alpha7 (and alpha6, before I rebased...)

- Removed the `xml_raw` feature, if you want the data, you can have it!
- `From<String>` and `From<serde_json::Error>` for `SplunkError`
- Added `SearchJob::map` which allows one to run functions over search results and get the return. I'm sure this is janky but it works.

## 0.1.3

- Cleaned up all the use of unwrap, more error handling etc.
- Updated dependencies
