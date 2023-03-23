//! Placeholder for now.

// #[macro_use]
// extern crate lazy_static;



#[cfg(feature="next")]
pub mod hec;
#[cfg(feature="next")]
pub mod search;

#[cfg(debug_assertions)]
mod tests;