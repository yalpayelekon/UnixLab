//! Path filter compilation for Quartz expressions.

/// One step in a compiled path filter.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Segment {
    /// Object field access by identifier key.
    Key(String),
    /// Array element access by non-negative index.
    Index(usize),
}

/// A compiled path filter.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Filter {
    segments: Vec<Segment>,
}

impl Filter {
    /// Returns the path segments in evaluation order.
    #[must_use]
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Parses a version 0.1 path filter.
    ///
    /// # Errors
    ///
    /// Returns a message when the filter syntax is not supported.
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut chars = input.chars().peekable();
        match chars.next() {
            Some('.') => {}
            Some(other) => return Err(format!("filter must start with '.', found '{other}'")),
            None => return Err("empty filter".to_owned()),
        }

        let mut segments = Vec::new();

        loop {
            match chars.peek().copied() {
                None => break,
                Some('[') => {
                    chars.next();
                    let index = parse_index(&mut chars)?;
                    expect_char(&mut chars, ']')?;
                    segments.push(Segment::Index(index));
                }
                Some('.') => {
                    chars.next();
                    match chars.peek().copied() {
                        Some('[') => {
                            chars.next();
                            let index = parse_index(&mut chars)?;
                            expect_char(&mut chars, ']')?;
                            segments.push(Segment::Index(index));
                        }
                        Some(ch) if is_ident_start(ch) => {
                            segments.push(Segment::Key(parse_ident(&mut chars)?));
                        }
                        Some(ch) => {
                            return Err(format!("expected key or index after '.', found '{ch}'"));
                        }
                        None => return Err("trailing '.' in filter".to_owned()),
                    }
                }
                Some(ch) if is_ident_start(ch) => {
                    segments.push(Segment::Key(parse_ident(&mut chars)?));
                }
                Some(ch) => return Err(format!("unexpected '{ch}' in filter")),
            }
        }

        Ok(Self { segments })
    }
}

fn parse_ident(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<String, String> {
    let mut key = String::new();
    let Some(first) = chars.next() else {
        return Err("expected identifier".to_owned());
    };
    if !is_ident_start(first) {
        return Err(format!("expected identifier start, found '{first}'"));
    }
    key.push(first);
    while let Some(ch) = chars.peek().copied() {
        if is_ident_continue(ch) {
            key.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    Ok(key)
}

fn parse_index(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Result<usize, String> {
    let mut digits = String::new();
    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    if digits.is_empty() {
        return Err("expected array index".to_owned());
    }
    if digits.len() > 1 && digits.starts_with('0') {
        return Err("array index must not have leading zeros".to_owned());
    }
    digits
        .parse::<usize>()
        .map_err(|_| format!("array index '{digits}' is out of range"))
}

fn expect_char(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    expected: char,
) -> Result<(), String> {
    match chars.next() {
        Some(ch) if ch == expected => Ok(()),
        Some(ch) => Err(format!("expected '{expected}', found '{ch}'")),
        None => Err(format!("expected '{expected}'")),
    }
}

const fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

const fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::{Filter, Segment};

    #[test]
    fn parses_identity() {
        let filter = Filter::parse(".").expect("identity");
        assert!(filter.segments().is_empty());
    }

    #[test]
    fn parses_nested_keys_and_index() {
        let filter = Filter::parse(".user.tags[1]").expect("path");
        assert_eq!(
            filter.segments(),
            [
                Segment::Key(String::from("user")),
                Segment::Key(String::from("tags")),
                Segment::Index(1),
            ]
        );
    }

    #[test]
    fn parses_root_index() {
        let filter = Filter::parse(".[0]").expect("root index");
        assert_eq!(filter.segments(), [Segment::Index(0)]);
    }

    #[test]
    fn rejects_empty_and_bad_syntax() {
        assert!(Filter::parse("").is_err());
        assert!(Filter::parse("name").is_err());
        assert!(Filter::parse(".foo.").is_err());
        assert!(Filter::parse(".[]").is_err());
        assert!(Filter::parse(".[01]").is_err());
        assert!(Filter::parse(".-1").is_err());
    }
}
