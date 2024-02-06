# Changelog

## 0.1.1-alpha6

- Removed the `xml_raw` feature, if you want the data, you can have it!
- `From<String>` and `From<serde_json::Error>` for `SplunkError`
- Added `SearchJob::map` which allows one to run functions over search results and get the return. I'm sure this is janky but it works.
