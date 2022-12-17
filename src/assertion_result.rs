use crate::ast::structs::Field;
use crate::ast::CodeSpan;
use miette::{ErrReport, SourceSpan};
use std::fmt;
use std::fmt::Formatter;
use std::path::Path;

#[derive(Debug)]
pub struct AssertionResult {
    pub expected: String,
    pub actual: Vec<ErrReport>,
}

impl AssertionResult {
    pub(crate) fn new() -> Self {
        AssertionResult {
            expected: "".to_owned(),
            actual: vec![],
        }
    }

    pub(crate) fn push_expected<S: AsRef<str>>(&mut self, expected: S) {
        self.expected.push_str(expected.as_ref());
    }

    pub(crate) fn push_actual<E: Into<ErrReport>>(&mut self, actual: E) {
        self.actual.push(actual.into());
    }
}

impl fmt::Display for AssertionResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Expected {} but found {} violations",
            self.expected,
            self.actual.len()
        )?;
        for report in &self.actual {
            writeln!(f, "{report:?}")?;
        }
        Ok(())
    }
}

pub(crate) fn get_code_sample_region(sample: &str, span: &CodeSpan) -> String {
    sample
        .lines()
        .enumerate()
        .filter(|(idx, _line)| idx + 1 >= span.start.line && idx < &span.end.line)
        .map(|(_, line)| line)
        .collect::<Vec<&str>>()
        .join("\n")
}

pub(crate) fn get_field_span(field: &Field, sample: &str) -> SourceSpan {
    field
        .name
        .as_ref()
        .and_then(|name| sample.find(name).map(|start| (start, name.len())))
        .map(Into::into)
        .unwrap_or_else(|| SourceSpan::from(field.span))
}

pub(crate) fn get_relative_location(location: &Path) -> String {
    let base = std::env::current_dir().expect("current dir");
    location
        .strip_prefix(base)
        .expect("should be in current_dir")
        .to_string_lossy()
        .to_string()
}
