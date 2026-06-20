//! Core KV3 value and object types.

use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum KV3Value {
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    Array(Vec<KV3Value>),
    HexArray(Vec<u8>),
    Object(KV3Object),
    Null,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KV3Object {
    #[cfg_attr(not(feature = "serde"), allow(dead_code))]
    pub(crate) fields: HashMap<String, KV3Value>,
}

impl KV3Object {
    pub(crate) fn new(fields: HashMap<String, KV3Value>) -> Self {
        Self { fields }
    }
}

/// Owned parser error with a rendered message and source span.
#[derive(Debug, Clone)]
pub struct KV3Error {
    pub message: String,
    pub span: Range<usize>,
}

impl fmt::Display for KV3Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse error at {}..{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for KV3Error {}
