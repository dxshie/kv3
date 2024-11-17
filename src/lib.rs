mod kv3_serde;
mod test;

use log::{debug, error, info};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    character::complete::{digit1, multispace0},
    combinator::{map, opt},
    multi::{many0, separated_list0},
    number::complete::recognize_float,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct KV3Object {
    fields: HashMap<String, KV3Value>,
}

pub fn parse_kv3(input: &str) -> IResult<&str, HashMap<String, KV3Value>> {
    info!("Parsing KV3 root...");

    // Skip initial comments and whitespace
    let (remaining, _) = skip_comments(input)?;

    // Parse the main object
    let (remaining, _) = preceded(multispace0, tag("{"))(remaining)?;
    let (remaining, kvs) = many0(preceded(multispace0, parse_key_value))(remaining)?;
    let (remaining, _) = preceded(multispace0, tag("}"))(remaining)?;

    debug!("Parsed KV3 root successfully: {:#?}", kvs);
    debug!("Remaining input: {:?}", remaining);

    Ok((remaining, kvs.into_iter().collect()))
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
fn skip_comments(input: &str) -> IResult<&str, ()> {
    map(
        many0(preceded(multispace0, parse_comment)), // Skip comments and surrounding whitespace
        |_| (),                                      // Ignore results
    )(input)
}

fn parse_number_or_float(input: &str) -> IResult<&str, KV3Value> {
    let (remaining, num_str) = recognize_float(input)?;
    if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
        let float_value = num_str.parse::<f64>().map_err(|_| {
            nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Float))
        })?;
        Ok((remaining, KV3Value::Double(float_value)))
    } else {
        let int_value = num_str.parse::<i64>().map_err(|_| {
            nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
        })?;
        Ok((remaining, KV3Value::Int(int_value)))
    }
}

fn parse_key_value(input: &str) -> IResult<&str, (String, KV3Value)> {
    info!("Parsing key-value pair...");
    let result = separated_pair(
        parse_key,
        preceded(multispace0, tag("=")),
        preceded(multispace0, parse_value),
    )(input);

    match &result {
        Ok((remaining, (key, value))) => {
            debug!("Parsed key-value pair: key = {}, value = {:?}", key, value);
            debug!("Remaining input: {:?}", remaining);
        }
        Err(e) => {
            error!("Error parsing key-value pair: {:?}", e);
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
            debug!("Parsed key: {}", key);
            debug!("Remaining input: {:?}", remaining);
        }
        Err(e) => {
            error!("Error parsing key: {:?}", e);
        }
    }

    result
}

fn parse_float(input: &str) -> IResult<&str, f64> {
    preceded(
        multispace0,
        map(recognize_float, |s: &str| s.parse::<f64>().unwrap()),
    )(input)
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

    match &result {
        Ok((remaining, bytes)) => {
            debug!("Parsed hex array: {:?}", bytes);
            debug!("Remaining input: {:?}", remaining);
        }
        Err(e) => {
            error!("Error parsing hex array: {:?}", e);
        }
    }

    result.map(|(remaining, bytes)| (remaining, KV3Value::HexArray(bytes)))
}

fn parse_value(input: &str) -> IResult<&str, KV3Value> {
    alt((
        map(tag("false"), |_| KV3Value::Bool(false)),
        map(tag("true"), |_| KV3Value::Bool(true)),
        map(tag("null"), |_| KV3Value::Null),
        map(parse_string, KV3Value::String),
        parse_array,
        parse_hex_array,
        parse_object,
        parse_number_or_float,
    ))(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    info!("Parsing string...");
    let result = delimited(tag("\""), take_until("\""), tag("\""))(input);

    if let Ok((remaining, string)) = &result {
        debug!("Parsed string: {}", string);
        debug!("Remaining input: {:?}", remaining);
    }

    result.map(|(remaining, s)| (remaining, s.to_string()))
}

fn parse_array(input: &str) -> IResult<&str, KV3Value> {
    info!("Parsing array...");

    // Parse elements separated by commas, and allow an optional trailing comma
    let parse_elements = separated_list0(
        preceded(multispace0, tag(",")),
        preceded(multispace0, parse_value),
    );
    let result = delimited(
        preceded(multispace0, tag("[")),
        terminated(parse_elements, opt(preceded(multispace0, tag(",")))), // Allow a trailing comma
        preceded(multispace0, tag("]")),
    )(input);

    match &result {
        Ok((remaining, elements)) => {
            debug!("Parsed array: {:?}", elements);
            debug!("Remaining input: {:?}", remaining);
        }
        Err(e) => {
            error!("Error parsing array: {:?}", e);
        }
    }

    result.map(|(remaining, fields)| (remaining, KV3Value::Array(fields)))
}

fn parse_object(input: &str) -> IResult<&str, KV3Value> {
    info!("Parsing object...");
    let parse_fields = many0(preceded(multispace0, parse_key_value));
    let result = delimited(
        preceded(multispace0, tag("{")),
        parse_fields,
        preceded(multispace0, tag("}")),
    )(input);

    match &result {
        Ok((remaining, fields)) => {
            debug!("Parsed object: {:?}", fields);
            debug!("Remaining input: {:?}", remaining);
        }
        Err(e) => {
            error!("Error parsing object: {:?}", e);
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

fn parse_number(input: &str) -> IResult<&str, i64> {
    info!("Parsing number...");
    let result = map(digit1, |s: &str| s.parse::<i64>().unwrap())(input);

    match &result {
        Ok((remaining, number)) => {
            debug!("Parsed number: {}", number);
            debug!("Remaining input: {:?}", remaining);
        }
        Err(e) => {
            error!("Error parsing number: {:?}", e);
        }
    }

    result
}
