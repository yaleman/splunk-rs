//! Want to search things? Here's your place.

use async_trait::async_trait;

#[macro_use]
pub mod searchjob;
pub mod kvstore;

pub use searchjob::{SearchJob, SearchResult};

use crate::client::SplunkClient;
use crate::errors::SplunkError;

#[async_trait]
/// Search trait for the Splunk client
pub trait SplunkSearch {
    /// Do a search and get a search job
    fn search(&self, query: &str) -> Result<SearchJob, String>;
    /// Do an "export" type search
    async fn export(&self) -> Result<(), SplunkError>;
}

#[async_trait]
impl SplunkSearch for SplunkClient {
    fn search(&self, query: &str) -> Result<SearchJob, String> {
        unimplemented!("this will run the query {}", query);
    }

    /// do an export-search - TODO
    async fn export(&self) -> Result<(), SplunkError> {
        unimplemented!();
    }
}
