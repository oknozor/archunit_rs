use crate::assertion_result::{get_code_sample_region, get_relative_location};
use crate::ast::{CodeSpan, Visibility};
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub(crate) enum EnumRuleViolation {
    #[error("Enum '{enum_name}' should derive '{trait_name}'")]
    #[diagnostic(help("Try adding `#[derive({trait_name})]` to `{enum_name}`"))]
    Derive {
        enum_name: String,
        trait_name: String,
        location: String,
        #[label("missing derive")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Enum '{enum_name}' should implement '{trait_name}'")]
    #[diagnostic(help(
        "Try writing the impl block `impl {trait_name} for {enum_name} {{ ... }}`"
    ))]
    Implement {
        enum_name: String,
        trait_name: String,
        location: String,
        #[label("missing impl")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Enum '{enum_name}' should implement or derive '{trait_name}'")]
    #[diagnostic(help(r#"Try adding `#[derive({trait_name})]` to `{enum_name}`
if that is not possible write the impl block `impl {trait_name} for {enum_name} {{ ... }}` manually"#))]
    ImplementOrDerive {
        enum_name: String,
        trait_name: String,
        location: String,
        #[label("missing derive or impl")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Enum '{enum_name}' should be private")]
    #[diagnostic(help("Try removing `pub` visibility"))]
    BePrivate {
        enum_name: String,
        location: String,
        #[label("should be private")]
        span: SourceSpan,
        vis: Visibility,
        #[source_code]
        src: NamedSource,
    },
    #[error("Enum '{enum_name}' should be public")]
    #[diagnostic(help("Try adding `pub` visibility"))]
    BePublic {
        enum_name: String,
        location: String,
        #[label("should be public")]
        span: SourceSpan,
        vis: Visibility,
        #[source_code]
        src: NamedSource,
    },
    #[error("Enum '{enum_name}' name should match {pattern}")]
    #[diagnostic(help("Try adding `pub` visibility"))]
    HaveNameMatching {
        enum_name: String,
        pattern: String,
        location: String,
        #[label("should be public")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
}

impl EnumRuleViolation {
    pub(crate) fn derive(
        span: CodeSpan,
        location: &PathBuf,
        enum_name: String,
        trait_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&enum_name).expect("enum name");
        let span = (start_hint, enum_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        EnumRuleViolation::Derive {
            enum_name,
            trait_name,
            location,
            span,
            src,
        }
    }

    pub(crate) fn implement(
        span: CodeSpan,
        location: &PathBuf,
        enum_name: String,
        trait_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&enum_name).expect("enum name");
        let span = (start_hint, enum_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        EnumRuleViolation::Implement {
            enum_name,
            trait_name,
            location,
            span,
            src,
        }
    }

    pub(crate) fn implement_or_derive(
        span: CodeSpan,
        location: &PathBuf,
        enum_name: String,
        trait_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&enum_name).expect("enum name");
        let span = (start_hint, enum_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        EnumRuleViolation::ImplementOrDerive {
            enum_name,
            trait_name,
            location,
            span,
            src,
        }
    }

    pub(crate) fn be_public(
        span: CodeSpan,
        location: &PathBuf,
        enum_name: String,
        vis: Visibility,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&enum_name).expect("enum name");
        let span = (start_hint, enum_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        EnumRuleViolation::BePublic {
            enum_name,
            location,
            span,
            vis,
            src,
        }
    }

    pub(crate) fn be_private(
        span: CodeSpan,
        location: &PathBuf,
        enum_name: String,
        vis: Visibility,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&enum_name).expect("enum name");
        let span = (start_hint, enum_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        EnumRuleViolation::BePrivate {
            enum_name,
            location,
            span,
            vis,
            src,
        }
    }
    pub(crate) fn have_simple(
        span: CodeSpan,
        pattern: String,
        location: &PathBuf,
        enum_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&enum_name).expect("enum name");
        let span = (start_hint, enum_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        EnumRuleViolation::HaveNameMatching {
            enum_name,
            pattern,
            location,
            span,
            src,
        }
    }
}
