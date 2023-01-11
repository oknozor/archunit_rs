use crate::assertion_result::{get_code_sample_region, get_relative_location};
use crate::ast::{CodeSpan, Visibility};
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub(crate) enum ModuleRuleViolation {
    #[error("Module '{module_name}' should be private")]
    #[diagnostic(help("Try removing `pub` visibility"))]
    BePrivate {
        module_name: String,
        location: String,
        #[label("should be private")]
        span: SourceSpan,
        vis: Visibility,
        #[source_code]
        src: NamedSource,
    },
    #[error("Module '{module_name}' should be public")]
    #[diagnostic(help("Try adding `pub` visibility"))]
    BePublic {
        module_name: String,
        location: String,
        #[label("should be public")]
        span: SourceSpan,
        vis: Visibility,
        #[source_code]
        src: NamedSource,
    },
    #[error("Module '{module_name}' name should match pattern '{pattern}'")]
    #[diagnostic(help("Try renaming '{module_name}' accordingly"))]
    HaveNameMatching {
        module_name: String,
        pattern: String,
        location: String,
        #[label("does not match")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Module '{module_name}' name should not match pattern '{pattern}'")]
    #[diagnostic(help("Try renaming '{module_name}' accordingly"))]
    DoesNotHaveNameMatching {
        module_name: String,
        pattern: String,
        location: String,
        #[label("does not match")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Module '{module_name}' name should only have dependency matching {pattern}")]
    #[diagnostic(help("Try removing usage of '{dependency}'"))]
    DependencyHaveNameMatching {
        module_name: String,
        dependency: String,
        pattern: String,
        location: String,
        #[label("name does not match")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Module '{module_name}' name should not have dependency matching {pattern}")]
    #[diagnostic(help("Try removing usage of '{dependency}'"))]
    DependencyDoesNotHaveNameMatching {
        module_name: String,
        dependency: String,
        pattern: String,
        location: String,
        #[label("name matches")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
}

impl ModuleRuleViolation {
    pub(crate) fn be_public(
        span: CodeSpan,
        location: &PathBuf,
        module_name: String,
        vis: Visibility,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        println!("{:?}", module_name);
        println!("{:?}", span);
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample
            .find(&module_name)
            .expect("Module name should be present in code sample");
        let span = (start_hint, module_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        ModuleRuleViolation::BePublic {
            module_name,
            location,
            span,
            vis,
            src,
        }
    }

    pub(crate) fn be_private(
        span: CodeSpan,
        location: &PathBuf,
        module_name: String,
        vis: Visibility,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample
            .find(&module_name)
            .expect("Module name should be present in code sample");
        let span = (start_hint, module_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        ModuleRuleViolation::BePrivate {
            module_name,
            location,
            span,
            vis,
            src,
        }
    }

    pub(crate) fn have_name_matching(
        span: CodeSpan,
        pattern: String,
        location: &PathBuf,
        module_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample
            .find(&module_name)
            .expect("Module name should be present in code sample");
        let span: SourceSpan = (start_hint, module_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);

        ModuleRuleViolation::HaveNameMatching {
            module_name,
            pattern,
            location,
            span,
            src,
        }
    }

    pub(crate) fn not_have_name_matching(
        span: CodeSpan,
        pattern: String,
        location: &PathBuf,
        module_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample
            .find(&module_name)
            .expect("Module name should be present in code sample");
        let span: SourceSpan = (start_hint, module_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);

        ModuleRuleViolation::DoesNotHaveNameMatching {
            module_name,
            pattern,
            location,
            span,
            src,
        }
    }

    pub(crate) fn only_have_dependencies_with_simple_name(
        span: CodeSpan,
        location: &PathBuf,
        module_name: String,
        pattern: String,
        dependency: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let span: SourceSpan = span.into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        ModuleRuleViolation::DependencyHaveNameMatching {
            module_name,
            dependency,
            pattern,
            location,
            span,
            src,
        }
    }

    pub(crate) fn only_have_dependencies_without_simple_name(
        span: CodeSpan,
        location: &PathBuf,
        module_name: String,
        pattern: String,
        dependency: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let span: SourceSpan = span.into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        ModuleRuleViolation::DependencyDoesNotHaveNameMatching {
            module_name,
            dependency,
            pattern,
            location,
            span,
            src,
        }
    }
}
