# Developer Notes

## Running tests

You need the following environment variables to run tests:

| env var               | example value                          |
|      ---              |                ---                     |
| SPLUNK_API_HOSTNAME   | "example.com"                          |
| SPLUNK_API_PORT       | "443"                                  |
| SPLUNK_HEC_HOSTNAME   | "example.com"                          |
| SPLUNK_HEC_TEST_INDEX | "test"                                 |
| SPLUNK_HEC_PORT       | "443"                                  |
| SPLUNK_HEC_TOKEN      | "12345678-0000-cafe-b33f-123456789012" |
| SPLUNK_USERNAME       | "happyuser"                            |
| SPLUNK_PASSWORD       | "hunter2*"                             |

## Skipping a test when it needs access to a live Splunk instance

Add this before your test, if the environment variable "CI" isn't set, or the feature "test_ci" isn't enabled, it won't run.

```rust
#[cfg_attr(feature="test_ci", ignore)]
```

To test this locally, run `cargo clean && CI=1 cargo test` or `cargo test --features test_ci`.
