use crate::assertion_result::{get_code_sample_region, get_field_span, get_relative_location};
use crate::ast::structs::Field;
use crate::ast::{CodeSpan, Visibility};
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum StructRuleViolation {
    #[error("Struct '{struct_name}' should derive '{trait_name}'")]
    #[diagnostic(help("Try adding `#[derive({trait_name})]` to `{struct_name}`"))]
    Derive {
        struct_name: String,
        trait_name: String,
        location: String,
        #[label("missing derive")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Struct '{struct_name}' should implement '{trait_name}'")]
    #[diagnostic(help(
        "Try writing the impl block `impl {trait_name} for {struct_name} {{ ... }}`"
    ))]
    Implement {
        struct_name: String,
        trait_name: String,
        location: String,
        #[label("missing impl")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Struct '{struct_name}' should implement or derive '{trait_name}'")]
    #[diagnostic(help(r#"Try adding `#[derive({trait_name})]` to `{struct_name}`
if that is not possible write the impl block `impl {trait_name} for {struct_name} {{ ... }}` manually"#))]
    ImplementOrDerive {
        struct_name: String,
        trait_name: String,
        location: String,
        #[label("missing derive or impl")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Struct '{struct_name}' should be private")]
    #[diagnostic(help("Try removing `pub` visibility"))]
    BePrivate {
        struct_name: String,
        location: String,
        #[label("should be private")]
        span: SourceSpan,
        vis: Visibility,
        #[source_code]
        src: NamedSource,
    },
    #[error("Struct '{struct_name}' should be public")]
    #[diagnostic(help("Try adding `pub` visibility"))]
    BePublic {
        struct_name: String,
        location: String,
        #[label("should be public")]
        span: SourceSpan,
        vis: Visibility,
        #[source_code]
        src: NamedSource,
    },
    #[error("Struct '{struct_name}' should not have public fields")]
    #[diagnostic(help("Try removing `pub` from field `{field_name}` visibility"))]
    OnlyHavePrivateFields {
        struct_name: String,
        field_name: String,
        location: String,
        #[label("public field")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Struct '{struct_name}' should not have private fields")]
    #[diagnostic(help("Try changing field visibility to `pub {field_name}`"))]
    OnlyHavePublicFields {
        struct_name: String,
        field_name: String,
        location: String,
        #[label("private field")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
    #[error("Struct '{struct_name}' name should match pattern: '{pattern}'")]
    #[diagnostic(help("Try renaming '{struct_name}' accordingly"))]
    HaveNameMatching {
        struct_name: String,
        pattern: String,
        location: String,
        #[label("missmatch")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource,
    },
}

impl StructRuleViolation {
    pub(crate) fn derive(
        span: CodeSpan,
        location: &PathBuf,
        struct_name: String,
        trait_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&struct_name).expect("struct name");
        let span = (start_hint, struct_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        StructRuleViolation::Derive {
            struct_name,
            trait_name,
            location,
            span,
            src,
        }
    }

    pub(crate) fn implement(
        span: CodeSpan,
        location: &PathBuf,
        struct_name: String,
        trait_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&struct_name).expect("struct name");
        let span = (start_hint, struct_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        StructRuleViolation::Implement {
            struct_name,
            trait_name,
            location,
            span,
            src,
        }
    }

    pub(crate) fn implement_or_derive(
        span: CodeSpan,
        location: &PathBuf,
        struct_name: String,
        trait_name: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&struct_name).expect("struct name");
        let span = (start_hint, struct_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        StructRuleViolation::ImplementOrDerive {
            struct_name,
            trait_name,
            location,
            span,
            src,
        }
    }

    pub(crate) fn be_public(
        span: CodeSpan,
        location: &PathBuf,
        struct_name: String,
        vis: Visibility,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&struct_name).expect("struct name");
        let span = (start_hint, struct_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        StructRuleViolation::BePublic {
            struct_name,
            location,
            span,
            vis,
            src,
        }
    }

    pub(crate) fn be_private(
        span: CodeSpan,
        location: &PathBuf,
        struct_name: String,
        vis: Visibility,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&struct_name).expect("struct name");
        let span = (start_hint, struct_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);
        StructRuleViolation::BePrivate {
            struct_name,
            location,
            span,
            vis,
            src,
        }
    }

    pub(crate) fn have_name_matching(
        span: CodeSpan,
        location: &PathBuf,
        struct_name: String,
        pattern: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        let start_hint = sample.find(&struct_name).expect("struct name");
        let span = (start_hint, struct_name.len()).into();
        let location = get_relative_location(location);
        let src = NamedSource::new(&location, sample);

        StructRuleViolation::HaveNameMatching {
            struct_name,
            pattern,
            location,
            span,
            src,
        }
    }

    pub(crate) fn only_have_public_fields(
        location: &PathBuf,
        struct_name: String,
        struct_fields: &[Field],
    ) -> Vec<Self> {
        let mut violations = vec![];
        let sample = fs::read_to_string(location).expect("path exists");
        let location = get_relative_location(location);

        for (idx, field) in struct_fields.iter().enumerate() {
            let span = field.span;
            let sample = get_code_sample_region(&sample, &span);
            let span = get_field_span(field, &sample);

            let src = NamedSource::new(&location, sample);
            if field.visibility != Visibility::Public {
                violations.push(StructRuleViolation::OnlyHavePublicFields {
                    struct_name: struct_name.clone(),
                    field_name: field.name.clone().unwrap_or_else(|| idx.to_string()),
                    location: location.clone(),
                    span,
                    src,
                })
            }
        }

        violations
    }

    pub(crate) fn only_have_private_fields(
        location: &PathBuf,
        struct_name: String,
        struct_fields: &[Field],
    ) -> Vec<Self> {
        let mut violations = vec![];
        let sample = fs::read_to_string(location).expect("path exists");
        let location = get_relative_location(location);

        for (idx, field) in struct_fields.iter().enumerate() {
            let span = field.span;
            let sample = get_code_sample_region(&sample, &span);
            let span = get_field_span(field, &sample);
            let src = NamedSource::new(&location, sample);

            if field.visibility == Visibility::Public {
                violations.push(StructRuleViolation::OnlyHavePrivateFields {
                    struct_name: struct_name.clone(),
                    field_name: field.name.clone().unwrap_or_else(|| idx.to_string()),
                    location: location.clone(),
                    span,
                    src,
                })
            }
        }

        violations
    }
}
