//! chumsky-based parser for the KV3 text format.

use std::collections::HashMap;

use chumsky::error::Rich;
use chumsky::extra;
use chumsky::prelude::*;

use crate::types::{KV3Error, KV3Object, KV3Value};

/// Parse a full KV3 document and return its top-level fields.
///
/// Returns a list of [`KV3Error`] values on failure, each carrying a rendered
/// message and a source span.
pub fn parse_kv3(input: &str) -> Result<HashMap<String, KV3Value>, Vec<KV3Error>> {
    document_parser()
        .parse(input)
        .into_result()
        .map_err(|errs| errs.into_iter().map(KV3Error::from_rich).collect())
}

impl KV3Error {
    fn from_rich(err: Rich<'_, char>) -> Self {
        let span = err.span();
        Self {
            message: err.to_string(),
            span: span.start..span.end,
        }
    }
}

type Extra<'src> = extra::Err<Rich<'src, char>>;

fn document_parser<'src>(
) -> impl Parser<'src, &'src str, HashMap<String, KV3Value>, Extra<'src>> {
    // --- whitespace + comments (pad) ---
    let line_comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .ignored();
    let block_comment = just("/*")
        .then(any().and_is(just("*/").not()).repeated())
        .then(just("*/"))
        .ignored();
    let xml_comment = just("<!--")
        .then(any().and_is(just("-->").not()).repeated())
        .then(just("-->"))
        .ignored();
    let comment = choice((line_comment, block_comment, xml_comment));
    let ws = any().filter(|c: &char| c.is_whitespace()).ignored();
    let pad = choice((ws, comment)).repeated();

    // --- identifier-style key ---
    let key = any()
        .filter(|c: &char| c.is_alphanumeric() || *c == '_')
        .repeated()
        .at_least(1)
        .to_slice()
        .map(|s: &str| s.to_string());

    // --- recursive value ---
    let value = recursive(|value| {
        let null = just("null").map(|_| KV3Value::Null);
        let bool_t = just("true").map(|_| KV3Value::Bool(true));
        let bool_f = just("false").map(|_| KV3Value::Bool(false));

        // numbers: optional sign, digits, optional .digits, optional e[+-]?digits
        let digits = any()
            .filter(|c: &char| c.is_ascii_digit())
            .repeated()
            .at_least(1);
        let frac = just('.').then(digits);
        let exp = one_of("eE").then(one_of("+-").or_not()).then(digits);
        let number = just('-')
            .or_not()
            .then(digits)
            .then(frac.or_not())
            .then(exp.or_not())
            .to_slice()
            .try_map(|s: &str, span| {
                if s.contains(['.', 'e', 'E']) {
                    s.parse::<f64>()
                        .map(KV3Value::Double)
                        .map_err(|e| Rich::custom(span, e.to_string()))
                } else {
                    s.parse::<i64>()
                        .map(KV3Value::Int)
                        .map_err(|e| Rich::custom(span, e.to_string()))
                }
            });

        // strings: prefer triple-quoted, fall back to single
        let multi_str = just("\"\"\"")
            .ignore_then(
                any()
                    .and_is(just("\"\"\"").not())
                    .repeated()
                    .to_slice(),
            )
            .then_ignore(just("\"\"\""))
            .map(|s: &str| KV3Value::String(s.to_string()));
        let single_str = just('"')
            .ignore_then(none_of('"').repeated().to_slice())
            .then_ignore(just('"'))
            .map(|s: &str| KV3Value::String(s.to_string()));
        let string = multi_str.or(single_str);

        // hex array: #[ ff fe ... ]
        let hex_array = just("#[")
            .ignore_then(none_of(']').repeated().to_slice())
            .then_ignore(just(']'))
            .map(|s: &str| {
                let bytes: Vec<u8> = s
                    .split_whitespace()
                    .filter_map(|h| u8::from_str_radix(h, 16).ok())
                    .collect();
                KV3Value::HexArray(bytes)
            });

        // array: `[ pad ( value pad (, pad)? )* ]`
        // Padding only at boundaries so empty `[]`/`[ ]` and trailing-comma cases
        // don't trigger consumption-after-failure in `repeated`/`separated_by`.
        let array_elem = value
            .clone()
            .then_ignore(pad)
            .then_ignore(just(',').then_ignore(pad).or_not());
        let array = pad
            .ignore_then(array_elem.repeated().collect::<Vec<_>>())
            .delimited_by(just('['), just(']'))
            .map(KV3Value::Array);

        // object: `{ pad ( kv pad )* }`
        let kv = key
            .then_ignore(pad)
            .then_ignore(just('='))
            .then_ignore(pad)
            .then(value.clone());
        let object = pad
            .ignore_then(kv.then_ignore(pad).repeated().collect::<Vec<_>>())
            .delimited_by(just('{'), just('}'))
            .map(|fields| KV3Value::Object(KV3Object::new(fields.into_iter().collect())));

        choice((
            array, hex_array, object, null, bool_t, bool_f, number, string,
        ))
    });

    // --- top-level: same shape as `object`, but returns a HashMap directly ---
    let top_kv = key
        .then_ignore(pad)
        .then_ignore(just('='))
        .then_ignore(pad)
        .then(value);

    pad.ignore_then(top_kv.then_ignore(pad).repeated().collect::<Vec<_>>())
        .delimited_by(just('{'), just('}'))
        .padded_by(pad)
        .then_ignore(end())
        .map(|kvs| kvs.into_iter().collect())
}
