//! Apply compiled filters to JSON values and encode results.

use std::io::{self, Write};

use serde_json::Value;

use crate::filter::{Filter, Segment};

/// Counts produced by a completed query run.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct QuerySummary {
    /// Number of input documents processed.
    pub documents: u64,
}

/// Applies `filter` to `value`, returning `null` for missing paths.
#[must_use]
pub fn apply(filter: &Filter, value: &Value) -> Value {
    let mut current = value;
    for segment in filter.segments() {
        match segment {
            Segment::Key(key) => match current {
                Value::Object(map) => match map.get(key) {
                    Some(next) => current = next,
                    None => return Value::Null,
                },
                _ => return Value::Null,
            },
            Segment::Index(index) => match current {
                Value::Array(items) => match items.get(*index) {
                    Some(next) => current = next,
                    None => return Value::Null,
                },
                _ => return Value::Null,
            },
        }
    }
    current.clone()
}

/// Writes one JSON value according to the compact/pretty setting.
///
/// # Errors
///
/// Returns an I/O error when writing fails.
pub fn write_value<W: Write>(value: &Value, compact: bool, writer: &mut W) -> io::Result<()> {
    let result = if compact {
        serde_json::to_writer(&mut *writer, value)
    } else {
        serde_json::to_writer_pretty(&mut *writer, value)
    };
    result.map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{apply, write_value};
    use crate::filter::Filter;
    use serde_json::json;

    #[test]
    fn identity_returns_whole_value() {
        let filter = Filter::parse(".").unwrap();
        let value = json!({"a": 1});
        assert_eq!(apply(&filter, &value), value);
    }

    #[test]
    fn nested_path_and_missing_key() {
        let value = json!({"user": {"name": "ada", "tags": ["a", "b"]}});
        let name = Filter::parse(".user.name").unwrap();
        assert_eq!(apply(&name, &value), json!("ada"));

        let tag = Filter::parse(".user.tags[1]").unwrap();
        assert_eq!(apply(&tag, &value), json!("b"));

        let missing = Filter::parse(".user.missing").unwrap();
        assert_eq!(apply(&missing, &value), json!(null));

        let out_of_range = Filter::parse(".user.tags[9]").unwrap();
        assert_eq!(apply(&out_of_range, &value), json!(null));
    }

    #[test]
    fn writes_compact_and_pretty() {
        let value = json!({"a": 1});
        let mut compact = Vec::new();
        write_value(&value, true, &mut compact).unwrap();
        assert_eq!(String::from_utf8(compact).unwrap(), "{\"a\":1}\n");

        let mut pretty = Vec::new();
        write_value(&value, false, &mut pretty).unwrap();
        assert_eq!(String::from_utf8(pretty).unwrap(), "{\n  \"a\": 1\n}\n");
    }
}
