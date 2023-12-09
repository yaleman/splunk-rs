//! Implementation of Search Jobs
//!
//!

use std::collections::HashMap;

use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::errors::SplunkError;

use super::SplunkClient;

// /// Error types for when we're trying to do searches
// #[derive(Debug)]
// pub enum SearchJobBuilderError {
//     /// While creating the search it failed in some way
//     CreateFailed {
//         /// The error message
//         message: String,
//     },
// }

#[derive(Debug, Clone)]
/// What kind of search mode we're using
pub enum SearchExecMode {
    /// Wait for the server to finish then respond
    Blocking,
    /// Stream the responses back straight away
    OneShot,
    /// "Normal" mode.
    Normal,
}
impl ToString for SearchExecMode {
    fn to_string(&self) -> String {
        match self {
            SearchExecMode::Blocking => "blocking",
            SearchExecMode::OneShot => "oneshot",
            SearchExecMode::Normal => "normal",
        }
        .to_string()
    }
}

#[derive(Clone, Debug)]
/// Set the search level
pub enum AdHocSearchLevel {
    /// Fast Mode - Field discovery off for event searches. No event or field data for stats searches.
    Fast,
    /// Smart Mode - Field discovery on for event searches. No event or field data for stats searches.
    Smart,
    /// Verbose Mode - All event & field data.
    Verbose,
}

impl ToString for AdHocSearchLevel {
    fn to_string(&self) -> String {
        match self {
            AdHocSearchLevel::Verbose => "verbose".to_string(),
            AdHocSearchLevel::Fast => "fast".to_string(),
            AdHocSearchLevel::Smart => "smart".to_string(),
        }
    }
}

#[allow(dead_code)]
#[allow(missing_docs)]
#[derive(Clone, Debug)]
/// What format the server will respond in
pub enum SearchOutputMode {
    Atom,
    Csv,
    Json,
    JsonCols,
    JsonRows,
    Raw,
    Xml,
}

impl ToString for SearchOutputMode {
    fn to_string(&self) -> String {
        match self {
            SearchOutputMode::Json => "json",
            SearchOutputMode::Atom => "atom",
            SearchOutputMode::Csv => "csv",
            SearchOutputMode::JsonCols => "json_cols",
            SearchOutputMode::JsonRows => "json_rows",
            SearchOutputMode::Raw => "raw",
            SearchOutputMode::Xml => "xml",
        }
        .to_string()
    }
}

#[derive(Clone, Debug)]
/// Builder object for a search job
pub struct SearchJobBuilder {
    /// The query string
    query: String,
    /// How many results we want back
    count: Option<u64>,
    /// Earliest time in the search, defaults to -24h
    /// The time string can be a UTC time (with fractional seconds), a relative time specifier (to now), or a formatted time string.
    earliest_time: String,
    /// Latest time, the time string can be a UTC time (with fractional seconds), a relative time specifier (to now), or a formatted time string.
    latest_time: String,
    /// Vec of fields we want to come back
    fields: Vec<String>,
    /// What Search level we're asking for, see [AdHocSearchLevel]
    adhoc_search_level: AdHocSearchLevel,
    /// Do we want previews, or should we wait
    allow_partial_results: bool,
    /// Automatically cancel the search after x seconds - set to 0 (default) for never.
    auto_cancel: u32,
    /// Automatically finalize and return the search after x events returned - set to 0 (default) for never.
    auto_finalize_ec: u32,
    /// Automatically pause the search after x seconds - set to 0 (default) for never.
    auto_pause: u32,
    /// See [SearchOutputMode]
    output_mode: SearchOutputMode,
    /// Custom parameter, see the doc examples for POST under `search/jobs` - <https://docs.splunk.com/Documentation/Splunk/latest/RESTREF/RESTsearch#search.2Fjobs>
    custom: Option<String>,
    /// Indicates whether lookups should be applied to events.
    /// Specifying true (the default) may slow searches significantly depending on the nature of the lookups.
    enable_lookups: bool,
    exec_mode: SearchExecMode,
    /// Specifies whether this search should cause (and wait depending on the value of sync_bundle_replication) for bundle synchronization with all search peers.
    force_bundle_replication: bool,
    /// Optional string to specify the search ID (`<sid>`). If unspecified, a random ID is generated.
    id: Option<String>,
    /// If you want to specify extra search options - see the details under `POST` in <https://docs.splunk.com/Documentation/Splunk/9.0.4/RESTREF/RESTsearch#search.2Fjobs>
    extra_options: HashMap<String, String>,
    timeout: u32,
}

impl Default for SearchJobBuilder {
    fn default() -> Self {
        let default_extra_options: HashMap<String, String> = HashMap::new();
        SearchJobBuilder {
            query: "".to_string(),
            count: Some(10000),
            earliest_time: "-24h".to_string(),
            latest_time: "now".to_string(),
            fields: vec![],
            adhoc_search_level: AdHocSearchLevel::Fast,
            allow_partial_results: true,
            auto_cancel: 0,
            auto_finalize_ec: 0,
            auto_pause: 0,
            output_mode: SearchOutputMode::Json,
            custom: None,
            enable_lookups: true,
            exec_mode: SearchExecMode::Normal,
            force_bundle_replication: false,
            id: None,
            extra_options: default_extra_options,
            timeout: 86400,
        }
    }
}

#[derive(Deserialize)]
/// Deserializer for Atom/XML response data
pub struct XMLResponseWithSid {
    #[allow(missing_docs)]
    pub response: XMLResponseSid,
}

#[derive(Deserialize)]
/// Deserializer for Atom/XML response data
pub struct XMLResponseSid {
    #[allow(missing_docs)]
    pub sid: String,
}

#[derive(Debug, Deserialize, Serialize)]
/// Deserializer for Atom/XML response data
pub struct SearchResult {
    preview: Option<bool>,
    offset: usize,
    lastrow: Option<bool>,
    result: serde_json::Value,
}

impl SearchJobBuilder {
    /// Consume the builder, start the job and return a search job object
    ///
    /// Options <https://docs.splunk.com/Documentation/Splunk/9.0.4/RESTREF/RESTsearch#search.2Fv2.2Fjobs.2Fexport>
    pub async fn create(self, client: &mut SplunkClient) -> Result<SearchJob, SplunkError> {
        // let endpoint = "/services/search/jobs/v2/export";
        let endpoint = "/services/search/jobs/export";
        let mut payload: HashMap<&str, String> = HashMap::new();

        self.extra_options.iter().for_each(|(key, value)| {
            payload.insert(key.as_str(), value.to_owned());
        });

        payload.insert("adhoc_search_level", self.adhoc_search_level.to_string());
        payload.insert(
            "allow_partial_results",
            self.allow_partial_results.to_string().to_ascii_lowercase(),
        );
        payload.insert("output_mode", self.output_mode.to_string());
        payload.insert("auto_cancel", format!("{}", self.auto_cancel));
        payload.insert("auto_finalize_ec", format!("{}", self.auto_finalize_ec));
        payload.insert("auto_pause", format!("{}", self.auto_pause));
        if let Some(custom) = self.custom {
            payload.insert("custom", custom);
        }
        payload.insert("earliest_time", self.earliest_time.clone());
        payload.insert("latest_time", self.latest_time.clone());
        payload.insert("timeout", self.timeout.to_string());
        payload.insert(
            "enable_lookups",
            self.enable_lookups.to_string().to_ascii_lowercase(),
        );
        payload.insert("exec_mode", self.exec_mode.to_string());
        payload.insert(
            "force_bundle_replication",
            self.force_bundle_replication
                .to_string()
                .to_ascii_lowercase(),
        );

        if let Some(id) = self.id {
            payload.insert("id", id);
        }

        // time to include the search
        payload.insert("search", self.query.clone());

        let result = match client.do_post(endpoint, payload).await {
            Err(err) => return Err(SplunkError::SearchCreationFailed(format!("{:?}", err))),
            Ok(val) => val,
        };

        Ok(SearchJob {
            query: self.query,
            count: self.count.unwrap(),
            earliest_time: self.earliest_time,
            latest_time: self.latest_time,
            fields: self.fields,
            exec_mode: self.exec_mode,
            sid: None,
            creation_response: result,
        })
    }

    /// sets adhoc_search_level
    pub fn adhoc_search_level(self, adhoc_search_level: AdHocSearchLevel) -> Self {
        Self {
            adhoc_search_level,
            ..self
        }
    }

    /// set the mode
    pub fn mode(self, exec_mode: SearchExecMode) -> Self {
        Self { exec_mode, ..self }
    }
}

/// A search job
pub struct SearchJob {
    /// The search query
    pub query: String,
    /// How many results we are asking for
    pub count: u64,
    /// What [SearchExecMode] to run in
    pub exec_mode: SearchExecMode,
    /// set earliest_time
    pub earliest_time: String,
    /// set latest_time
    pub latest_time: String,
    /// ask for a particular set of fields - leave empty for all
    pub fields: Vec<String>,
    /// set the search ID on creation
    pub sid: Option<String>,
    /// The raw `reqwest::Response` object
    pub creation_response: Response,
}

#[allow(unused_macros)]
macro_rules! get_lines {
    ($stream:expr) => {
        StreamReader::new($stream.map_err(convert_err)).lines()
    };
}

impl SearchJob {
    /// Defaults to 10000 results, last 24 hours -> now(),
    pub fn create(query: impl Into<String>) -> SearchJobBuilder {
        SearchJobBuilder {
            query: query.into(),
            ..Default::default()
        }
    }
}
