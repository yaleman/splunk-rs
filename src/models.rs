//! Shared data models used by the client, HEC and search modules.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Request payload types.
pub mod requests {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    /// Basic auth login payload
    pub struct BasicAuth<'a> {
        /// username
        pub username: &'a str,
        /// password
        pub password: &'a str,
    }
}

/// Response payload types.
pub mod responses {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    /// This is the "generator" element in API Responses
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ApiResponseGenerator {
        /// The Splunk build ID
        pub build: String,
        /// The Splunk version number
        pub version: String,
    }

    /// This is the "paging" element in API Responses
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ApiResponsePaging {
        /// Total possible responses
        pub total: u32,
        #[serde(rename = "perPage")]
        /// Number returned per page
        pub per_page: u32,
        /// Current request offset
        pub offset: u32,
    }

    impl ApiResponsePaging {
        /// Is there more pages?
        pub fn has_more(&self) -> bool {
            if self.offset > self.total {
                false
            } else {
                (self.total - self.per_page) > self.offset
            }
        }
    }

    /// Trying to capture an API response as a struct!
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ApiResponse {
        /// Links to other endpoints
        pub links: HashMap<String, String>,
        /// The endpoint which generated this response
        pub origin: String,
        /// The timestamp of the request
        pub updated: String, // TODO: parse this into an offsetdatetime
        /// Splunk version/build that generated this response
        pub generator: Option<ApiResponseGenerator>,
        /// The results
        pub entry: Vec<Value>,
        /// Information/error messages in your response
        pub messages: Vec<Value>,
        /// Details of where you are in the response set
        pub paging: Option<ApiResponsePaging>,
    }

    impl ApiResponse {
        /// Check that the paging indicates we have more results - if there's no paging data in the response, then you get a false regardless.
        pub fn paging_has_more(&self) -> bool {
            if let Some(paging) = &self.paging {
                paging.has_more()
            } else {
                false
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The current auth method for the search client
pub enum AuthenticationMethod {
    /// Basic auth
    Basic {
        /// username
        username: String,
        ///password
        password: String,
    },
    /// Token auth
    Token {
        /// token auth
        token: String,
    },
    /// Cookie based
    Cookie {
        /// cookie store
        cookie: HashMap<String, String>,
    },
    /// we haven't set it yet
    Unknown,
}

#[derive(Debug, Deserialize, Serialize)]
/// the current auth mode - you can auth with username/password then get a cookie and go from there
pub enum AuthenticatedSessionMode {
    /// cookie auth
    Cookie {
        /// cookie value
        value: HashMap<String, String>,
    },
    /// token auth
    Token(
        /// the token
        String,
    ),
    /// we haven't set it yet
    Unset,
}
