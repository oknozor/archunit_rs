use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct AssertionResult {
    pub expected: String,
    pub actual: String,
}

impl AssertionResult {
    pub(crate) fn new() -> Self {
        AssertionResult {
            expected: "".to_string(),
            actual: "".to_string(),
        }
    }

    pub(crate) fn push_expected<S: AsRef<str>>(&mut self, expected: S) {
        self.expected.push_str(expected.as_ref());
    }

    pub(crate) fn push_actual<S: AsRef<str>>(&mut self, actual: S) {
        self.actual.push_str(actual.as_ref());
    }
}

impl fmt::Display for AssertionResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Expected {}\nbut {}", self.expected, self.actual)
    }
}
