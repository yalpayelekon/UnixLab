//! Address compilation and evaluation for Reed scripts.

use regex::Regex;

/// Line state used while evaluating addresses.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct LineState {
    /// One-based line number in the continuous input stream.
    pub line_no: u64,
    /// Whether this is the final line of the input stream.
    pub is_last: bool,
}

/// A single address selector.
#[derive(Debug)]
pub enum Address {
    /// Absolute one-based line number.
    Line(u64),
    /// The last input line.
    Last,
    /// Lines matching a regular expression.
    Regex(Regex),
}

impl Address {
    /// Reports whether this address selects the current line.
    #[must_use]
    pub fn matches(&self, line: &str, state: LineState) -> bool {
        match self {
            Self::Line(number) => state.line_no == *number,
            Self::Last => state.is_last,
            Self::Regex(pattern) => pattern.is_match(line),
        }
    }

    /// Reports whether evaluating this address needs last-line knowledge.
    #[must_use]
    pub const fn needs_last_line(&self) -> bool {
        matches!(self, Self::Last)
    }
}

/// Zero, one, or two addresses attached to a command.
#[derive(Debug)]
pub enum AddressSpec {
    /// Applies to every line.
    All,
    /// Applies when a single address matches.
    Single(Address),
    /// Applies for an inclusive range between start and end.
    Range {
        /// First line of the range.
        start: Address,
        /// Last line of the range.
        end: Address,
    },
}

impl AddressSpec {
    /// Reports whether evaluating this address needs last-line knowledge.
    #[must_use]
    pub fn needs_last_line(&self) -> bool {
        match self {
            Self::All => false,
            Self::Single(address) => address.needs_last_line(),
            Self::Range { start, end } => start.needs_last_line() || end.needs_last_line(),
        }
    }
}

/// Tracks whether a two-address range is currently active.
#[derive(Debug, Default)]
pub struct RangeState {
    active: bool,
}

impl RangeState {
    /// Creates an inactive range tracker.
    #[must_use]
    pub const fn new() -> Self {
        Self { active: false }
    }

    /// Returns whether `spec` selects the current line, updating range state.
    pub fn matches(&mut self, spec: &AddressSpec, line: &str, state: LineState) -> bool {
        match spec {
            AddressSpec::All => true,
            AddressSpec::Single(address) => address.matches(line, state),
            AddressSpec::Range { start, end } => {
                if self.active {
                    if end.matches(line, state) {
                        self.active = false;
                    }
                    true
                } else if start.matches(line, state) {
                    if !end.matches(line, state) {
                        self.active = true;
                    }
                    true
                } else {
                    false
                }
            }
        }
    }
}

/// Parses one address from `input`, returning the address and remaining text.
///
/// # Errors
///
/// Returns a descriptive message when the address is malformed or the regex
/// cannot be compiled.
pub fn parse_address(input: &str) -> Result<(Address, &str), String> {
    let input = input.trim_start();
    if input.is_empty() {
        return Err("expected address".to_owned());
    }

    let mut chars = input.char_indices();
    let Some((_, first)) = chars.next() else {
        return Err("expected address".to_owned());
    };

    if first == '$' {
        return Ok((Address::Last, &input[1..]));
    }

    if first.is_ascii_digit() {
        let mut end = 1;
        for (index, ch) in chars {
            if ch.is_ascii_digit() {
                end = index + ch.len_utf8();
            } else {
                break;
            }
        }
        let number = input[..end]
            .parse::<u64>()
            .map_err(|_| format!("invalid line address '{}'", &input[..end]))?;
        if number == 0 {
            return Err("line addresses are one-based".to_owned());
        }
        return Ok((Address::Line(number), &input[end..]));
    }

    if first == '/' {
        let (pattern, rest) = take_delimited(&input[1..], '/')?;
        let regex = Regex::new(&pattern).map_err(|error| error.to_string())?;
        return Ok((Address::Regex(regex), rest));
    }

    Err(format!("invalid address starting with '{first}'"))
}

fn take_delimited(input: &str, delimiter: char) -> Result<(String, &str), String> {
    let mut pattern = String::new();
    let mut chars = input.char_indices();
    while let Some((index, ch)) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some((_, escaped)) => {
                    if escaped == delimiter {
                        pattern.push(delimiter);
                    } else {
                        pattern.push('\\');
                        pattern.push(escaped);
                    }
                }
                None => return Err("trailing backslash in address pattern".to_owned()),
            }
            continue;
        }
        if ch == delimiter {
            return Ok((pattern, &input[index + delimiter.len_utf8()..]));
        }
        pattern.push(ch);
    }
    Err(format!("unclosed '{delimiter}' address"))
}

#[cfg(test)]
mod tests {
    use super::{Address, AddressSpec, LineState, RangeState, parse_address};

    #[test]
    fn parses_line_last_and_regex_addresses() {
        let (line, rest) = parse_address("12p").expect("line address");
        assert!(matches!(line, Address::Line(12)));
        assert_eq!(rest, "p");

        let (last, rest) = parse_address("$d").expect("last address");
        assert!(matches!(last, Address::Last));
        assert_eq!(rest, "d");

        let (regex, rest) = parse_address("/foo\\/bar/p").expect("regex address");
        assert!(matches!(regex, Address::Regex(_)));
        assert_eq!(rest, "p");
    }

    #[test]
    fn range_activates_until_end_address() {
        let start = Address::Line(2);
        let end = Address::Line(4);
        let spec = AddressSpec::Range { start, end };
        let mut state = RangeState::new();

        assert!(!state.matches(
            &spec,
            "a",
            LineState {
                line_no: 1,
                is_last: false
            }
        ));
        assert!(state.matches(
            &spec,
            "b",
            LineState {
                line_no: 2,
                is_last: false
            }
        ));
        assert!(state.matches(
            &spec,
            "c",
            LineState {
                line_no: 3,
                is_last: false
            }
        ));
        assert!(state.matches(
            &spec,
            "d",
            LineState {
                line_no: 4,
                is_last: false
            }
        ));
        assert!(!state.matches(
            &spec,
            "e",
            LineState {
                line_no: 5,
                is_last: true
            }
        ));
    }
}
