#![deny(clippy::all)]
#![deny(missing_docs)]

//! Async adapters for nom streaming parsers

mod stream;

pub use stream::*;
