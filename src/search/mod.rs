//! Want to search things? Here's your place.

#[macro_use]
pub mod searchjob;
pub mod kvstore;

pub use searchjob::{SearchJob, SearchResult};
