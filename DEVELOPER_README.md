# Developer Notes

## Skipping a test when it needs access to a live Splunk instance

Add this before your test, if the environment variable "CI" isn't set, or the feature "test_ci" isn't enabled, it won't run.

```rust
#[cfg_attr(feature="test_ci", ignore)]
```

To test this locally, run `cargo clean && CI=1 cargo test` or `cargo test --features test_ci`.
