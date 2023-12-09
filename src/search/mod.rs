//! Want to search things? Here's your place.

use async_trait::async_trait;

#[macro_use]
pub mod searchjob;
pub mod kvstore;

pub use searchjob::{SearchJob, SearchResult};

use crate::client::SplunkClient;
use crate::errors::SplunkError;

#[async_trait]
trait SplunkSearch {
    fn search(&self, query: &str) -> Result<SearchJob, String>;
    async fn export() -> Result<(), SplunkError>;
}

#[async_trait]
impl SplunkSearch for SplunkClient {
    fn search(&self, query: &str) -> Result<SearchJob, String> {
        unimplemented!("this will run the query {}", query);
    }

    /// do an export-search - TODO
    async fn export() -> Result<(), SplunkError> {
        unimplemented!();
    }
}
