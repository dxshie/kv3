//! `Display` implementations that pretty-print KV3 values as KV3 text.

use std::fmt;

use crate::types::{KV3Object, KV3Value};

const INDENT: &str = "    ";
const HEX_BYTES_PER_LINE: usize = 16;

impl fmt::Display for KV3Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_value(f, self, 0)
    }
}

impl fmt::Display for KV3Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_object(f, self, 0)
    }
}

fn write_value(f: &mut fmt::Formatter<'_>, v: &KV3Value, indent: usize) -> fmt::Result {
    match v {
        KV3Value::Null => f.write_str("null"),
        KV3Value::Bool(b) => write!(f, "{}", b),
        KV3Value::Int(i) => write!(f, "{}", i),
        KV3Value::Double(d) => write_double(f, *d),
        KV3Value::String(s) => write_string(f, s),
        KV3Value::Array(items) => write_array(f, items, indent),
        KV3Value::HexArray(bytes) => write_hex_array(f, bytes, indent),
        KV3Value::Object(obj) => write_object(f, obj, indent),
    }
}

fn write_double(f: &mut fmt::Formatter<'_>, d: f64) -> fmt::Result {
    // Round-trip safety: always render with a `.` (or exponent) so the parser
    // takes it as a Double rather than an Int.
    if d.is_finite() && d.fract() == 0.0 && d.abs() < 1e16 {
        write!(f, "{:.1}", d)
    } else {
        let s = format!("{}", d);
        if s.contains(['.', 'e', 'E']) {
            f.write_str(&s)
        } else {
            write!(f, "{}.0", d)
        }
    }
}

fn write_string(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    // No escape support in either parser or printer; fall back to triple-quoted
    // when the value would otherwise be ambiguous.
    if s.contains('\n') || s.contains('"') {
        f.write_str("\"\"\"")?;
        f.write_str(s)?;
        f.write_str("\"\"\"")
    } else {
        f.write_str("\"")?;
        f.write_str(s)?;
        f.write_str("\"")
    }
}

fn write_indent(f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
    for _ in 0..indent {
        f.write_str(INDENT)?;
    }
    Ok(())
}

fn write_array(f: &mut fmt::Formatter<'_>, items: &[KV3Value], indent: usize) -> fmt::Result {
    if items.is_empty() {
        return f.write_str("[]");
    }
    f.write_str("[\n")?;
    for item in items {
        write_indent(f, indent + 1)?;
        write_value(f, item, indent + 1)?;
        f.write_str(",\n")?;
    }
    write_indent(f, indent)?;
    f.write_str("]")
}

fn write_hex_array(f: &mut fmt::Formatter<'_>, bytes: &[u8], indent: usize) -> fmt::Result {
    if bytes.is_empty() {
        return f.write_str("#[]");
    }
    f.write_str("#[\n")?;
    for (i, b) in bytes.iter().enumerate() {
        let column = i % HEX_BYTES_PER_LINE;
        if column == 0 {
            write_indent(f, indent + 1)?;
        } else {
            f.write_str(" ")?;
        }
        write!(f, "{:02X}", b)?;
        if column == HEX_BYTES_PER_LINE - 1 {
            f.write_str("\n")?;
        }
    }
    if !bytes.len().is_multiple_of(HEX_BYTES_PER_LINE) {
        f.write_str("\n")?;
    }
    write_indent(f, indent)?;
    f.write_str("]")
}

fn write_object(f: &mut fmt::Formatter<'_>, obj: &KV3Object, indent: usize) -> fmt::Result {
    if obj.fields.is_empty() {
        return f.write_str("{}");
    }
    f.write_str("{\n")?;
    // Sort keys for deterministic output (HashMap iteration order is unstable).
    let mut keys: Vec<&String> = obj.fields.keys().collect();
    keys.sort();
    for key in keys {
        write_indent(f, indent + 1)?;
        f.write_str(key)?;
        f.write_str(" = ")?;
        write_value(f, &obj.fields[key], indent + 1)?;
        f.write_str("\n")?;
    }
    write_indent(f, indent)?;
    f.write_str("}")
}
