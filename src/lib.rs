//! # kv3
//!
//! A Rust crate for parsing Valve's KeyValues3 (KV3) format.
//!
//! This crate provides functionality to parse and serialize the KeyValues3 format used by Valve in their games and tools.
//!
//! ## Features
//!
//! - **Parsing**: Convert KV3-formatted strings into Rust data structures.
//! - **Support for Comments**: Handles single-line (`//`), multi-line (`/* ... */`), and XML-style (`<!-- ... -->`) comments.
//! - **Support for Multiline Strings**: Parses multiline strings enclosed in triple double-quotes (`"""`).
//! - **Handles Various Data Types**: Supports booleans, integers, floats, strings, arrays, hex arrays, objects, and null values.
//! - **Rich Error Reporting**: Built on the [`chumsky`](https://docs.rs/chumsky) parser combinator library, producing errors with source spans and "expected … found …" diagnostics.
//!
//! ## Example
//!
//! ```rust
//! use kv3::parse_kv3;
//!
//! let input = r#"
//! {
//!     // comment
//!     number = 5
//!     floating = 5.0
//!     array = []
//!     obj = {}
//!     string = "asd"
//!     multiLineStringValue = """
//! First line of a multi-line string literal.
//! Second line of a multi-line string literal.
//! """
//! }
//! "#;
//!
//! match parse_kv3(input) {
//!     Ok(kvs) => {
//!         println!("Parsed KV3: {:#?}", kvs);
//!     }
//!     Err(errors) => {
//!         for e in errors {
//!             eprintln!("{}", e);
//!         }
//!     }
//! }
//! ```
//!
//! ## KeyValues3 Format
//!
//! For more information about the KeyValues3 format, please refer to the [Valve Developer Community Wiki](https://developer.valvesoftware.com/wiki/KeyValues3).
//!
//! ## License
//!
//! This project is licensed under the GPL-3.0-only License.

mod format;
mod parser;
mod types;

#[cfg(feature = "serde")]
pub mod kv3_serde;

#[cfg(test)]
mod test;

pub use parser::parse_kv3;
pub use types::{KV3Error, KV3Object, KV3Value};

#[cfg(feature = "serde")]
pub use kv3_serde::{serde_kv3, to_kv3_string, KV3SerError};
