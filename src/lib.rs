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
//! - **Customizable Parsing**: Built using the [`nom`](https://github.com/Geal/nom) parser combinator library for flexibility.
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
//!     Ok((_, kvs)) => {
//!         println!("Parsed KV3: {:#?}", kvs);
//!     }
//!     Err(e) => {
//!         eprintln!("Error parsing KV3: {:?}", e);
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
//! This project is licensed under the MIT License.
//!

#[cfg(feature = "serde")]
pub mod kv3_serde;

mod test;

use log::{debug, error, info};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::multispace1,
    combinator::{map, opt},
    multi::{many0, separated_list0},
    number::complete::recognize_float,
    sequence::{delimited, pair, preceded, separated_pair},
    IResult,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "serde")]
#[derive(Debug, Serialize)]
pub enum KV3Value {
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    Array(Vec<KV3Value>),
    HexArray(Vec<u8>), // New variant for hexadecimal arrays
    Object(KV3Object),
    Null,
}

#[cfg(not(feature = "serde"))]
#[derive(Debug)]
pub enum KV3Value {
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
    Array(Vec<KV3Value>),
    HexArray(Vec<u8>), // New variant for hexadecimal arrays
    Object(KV3Object),
    Null,
}

#[cfg(feature = "serde")]
#[derive(Debug, Serialize, Deserialize)]
pub struct KV3Object {
    fields: HashMap<String, KV3Value>,
}

#[cfg(not(feature = "serde"))]
#[allow(dead_code)]
#[derive(Debug)]
pub struct KV3Object {
    fields: HashMap<String, KV3Value>,
}

pub fn parse_kv3(input: &str) -> IResult<&str, HashMap<String, KV3Value>> {
    info!("Parsing KV3 root...");

    let (remaining, _) = ws(tag("{"))(input)?;
    let (remaining, kvs) = many0(ws(parse_key_value))(remaining)?;
    let (remaining, _) = ws(tag("}"))(remaining)?;

    debug!(
        "Parsed KV3 root successfully: {:?}",
        truncate_str(&format!("{:#?}", kvs), 100)
    );
    debug!("Remaining input: {}", truncate_str(remaining, 100));

    Ok((remaining, kvs.into_iter().collect()))
}

fn truncate_str(input: &str, max_length: usize) -> String {
    if input.len() > max_length {
        format!("{}... (truncated)", &input[..max_length])
    } else {
        input.to_string()
    }
}

fn parse_comment(input: &str) -> IResult<&str, ()> {
    // Parse single-line comments (// ...)
    let single_line = map(
        preceded(tag("//"), take_until("\n")),
        |_| (), // Ignore content
    );

    // Parse multi-line comments (/* ... */)
    let multi_line = map(
        delimited(tag("/*"), take_until("*/"), tag("*/")),
        |_| (), // Ignore content
    );

    // Parse XML-style comments (<!-- ... -->)
    let xml_style = map(
        delimited(tag("<!--"), take_until("-->"), tag("-->")),
        |_| (), // Ignore content
    );

    // Combine all comment formats
    alt((single_line, multi_line, xml_style))(input)
}

fn skip_comments_and_whitespace(input: &str) -> IResult<&str, ()> {
    map(
        many0(alt((map(multispace1, |_| ()), parse_comment))),
        |_| (),
    )(input)
}

fn ws<'a, F, O>(inner: F) -> impl Fn(&'a str) -> IResult<&'a str, O>
where
    F: 'a + Fn(&'a str) -> IResult<&'a str, O>,
{
    move |input: &str| {
        let (input, _) = skip_comments_and_whitespace(input)?;
        let (input, res) = inner(input)?;
        let (input, _) = skip_comments_and_whitespace(input)?;
        Ok((input, res))
    }
}

fn parse_number_or_float(input: &str) -> IResult<&str, KV3Value> {
    let input = input.trim_start(); // Trim leading whitespace

    let (remaining, num_str) = recognize_float(input)?;
    if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
        // Parse as float
        num_str
            .parse::<f64>()
            .map(|v| (remaining, KV3Value::Double(v)))
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Float))
            })
    } else {
        // Parse as integer
        num_str
            .parse::<i64>()
            .map(|v| (remaining, KV3Value::Int(v)))
            .map_err(|_| {
                nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
            })
    }
}

fn parse_key_value(input: &str) -> IResult<&str, (String, KV3Value)> {
    debug!("Parsing key-value pair...");
    let result = separated_pair(ws(parse_key), ws(tag("=")), ws(parse_value))(input);

    match &result {
        Ok((remaining, (key, value))) => {
            debug!(
                "Parsed key-value pair: key = {}, value = {:?}",
                truncate_str(key, 50),
                truncate_str(&format!("{:?}", value), 50)
            );
            debug!(
                "Remaining input after key-value: {}",
                truncate_str(remaining, 200)
            );
        }
        Err(e) => {
            error!(
                "Error parsing key-value pair: {:?}",
                truncate_str(e.to_string().as_str(), 200)
            );
        }
    }

    result
}

fn parse_key(input: &str) -> IResult<&str, String> {
    info!("Parsing key...");
    let result = map(
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
        |s: &str| s.to_string(),
    )(input);

    match &result {
        Ok((remaining, key)) => {
            debug!("Parsed key: {}", truncate_str(key, 50));
            debug!("Remaining input: {}", truncate_str(remaining, 200));
        }
        Err(e) => {
            error!(
                "Error parsing key: {:?}",
                truncate_str(e.to_string().as_str(), 200)
            );
        }
    }

    result
}

fn parse_value(input: &str) -> IResult<&str, KV3Value> {
    alt((
        parse_array, // Prioritize array parsing
        parse_hex_array,
        parse_object,
        map(tag("false"), |_| KV3Value::Bool(false)),
        map(tag("true"), |_| KV3Value::Bool(true)),
        map(tag("null"), |_| KV3Value::Null),
        parse_number_or_float,               // Parse numbers
        map(parse_string, KV3Value::String), // Parse strings last
    ))(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    info!("Parsing string...");

    // Parser for multiline strings
    let parse_multiline_string = delimited(tag("\"\"\""), take_until("\"\"\""), tag("\"\"\""));

    // Parser for single-line strings
    let parse_single_line_string = delimited(tag("\""), take_until("\""), tag("\""));

    // Try to parse a multiline string first, then a single-line string
    let result = alt((parse_multiline_string, parse_single_line_string))(input);

    if let Ok((remaining, string)) = &result {
        debug!("Parsed string: {}", truncate_str(string, 50));
        debug!("Remaining input: {}", truncate_str(remaining, 200));
    }

    result.map(|(remaining, s)| (remaining, s.to_string()))
}

fn parse_array(input: &str) -> IResult<&str, KV3Value> {
    info!("Parsing array...");

    // Parse elements as KV3 values, separated by commas
    let parse_elements = separated_list0(ws(tag(",")), ws(parse_value));
    let mut array_parser = delimited(
        ws(tag("[")), // Opening bracket
        map(
            // Wrap parsed elements in a vector
            pair(parse_elements, opt(ws(tag(",")))), // Allow optional trailing comma
            |(elements, _)| elements,                // Discard the optional trailing comma
        ),
        ws(tag("]")), // Closing bracket
    );

    let result = array_parser(input);

    // Log results or errors
    match &result {
        Ok((remaining, elements)) => {
            debug!(
                "Parsed array with {} elements: {:?}",
                elements.len(),
                truncate_str(&format!("{:?}", elements), 100)
            );
            debug!(
                "Remaining input after array: {}",
                truncate_str(remaining, 200)
            );
        }
        Err(e) => {
            error!(
                "Error parsing array: {:?}",
                truncate_str(e.to_string().as_str(), 200)
            );
        }
    }

    result.map(|(remaining, elements)| (remaining, KV3Value::Array(elements)))
}

fn parse_hex_array(input: &str) -> IResult<&str, KV3Value> {
    info!("Parsing hex array...");
    let result = delimited(
        tag("#["),
        map(take_until("]"), |content: &str| {
            content
                .split_whitespace() // Split into hex pairs
                .filter_map(|hex| u8::from_str_radix(hex, 16).ok()) // Parse as u8
                .collect::<Vec<u8>>() // Collect into a vector
        }),
        tag("]"),
    )(input);

    result.map(|(remaining, bytes)| (remaining, KV3Value::HexArray(bytes)))
}

fn parse_object(input: &str) -> IResult<&str, KV3Value> {
    info!("Parsing object...");
    let parse_fields = many0(ws(parse_key_value));
    let result = delimited(ws(tag("{")), parse_fields, ws(tag("}")))(input);

    match &result {
        Ok((remaining, key)) => {
            debug!("Parsed object: {:?}", key);
            debug!("Remaining input: {}", truncate_str(remaining, 200));
        }
        Err(e) => {
            error!(
                "Error parsing key: {:?}",
                truncate_str(e.to_string().as_str(), 200)
            );
        }
    }

    result.map(|(remaining, fields)| {
        (
            remaining,
            KV3Value::Object(KV3Object {
                fields: fields.into_iter().collect(),
            }),
        )
    })
}
