//! Regular-expression compilation and matching.

use regex::{Matches, Regex};

/// A compiled Needle search pattern.
#[derive(Debug)]
pub struct Matcher {
    regex: Regex,
}

impl Matcher {
    /// Compiles a Unicode-aware regular expression.
    ///
    /// # Errors
    ///
    /// Returns the regex engine's parse error when `pattern` is invalid.
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Regex::new(pattern).map(|regex| Self { regex })
    }

    /// Reports whether the line contains at least one match.
    #[must_use]
    pub fn is_match(&self, line: &str) -> bool {
        self.regex.is_match(line)
    }

    /// Iterates over non-overlapping matches in the line.
    #[must_use]
    pub fn matches<'matcher, 'text>(&'matcher self, line: &'text str) -> Matches<'matcher, 'text> {
        self.regex.find_iter(line)
    }
}

#[cfg(test)]
mod tests {
    use super::Matcher;

    #[test]
    fn matches_unicode_text() {
        let matcher = Matcher::new(r"\p{Greek}+").expect("pattern should compile");

        assert!(matcher.is_match("value: λογος"));
        assert!(!matcher.is_match("value: needle"));
    }

    #[test]
    fn rejects_invalid_pattern() {
        let error = Matcher::new("(").expect_err("pattern should be invalid");

        assert!(error.to_string().contains("unclosed group"));
    }
}
