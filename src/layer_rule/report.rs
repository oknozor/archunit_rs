use crate::assertion_result::get_code_sample_region;
use crate::ast::CodeSpan;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Forbidden access to layer '{layer}' in {accessed_in}")]
#[diagnostic(help("Try refactoring your code to remove usage of '{layer_module}'"))]
pub struct ForbiddenLayerAccess {
    layer: String,
    layer_module: String,
    accessed_in: String,
    #[source_code]
    src: NamedSource,
    #[label("Forbidden usage")]
    span: SourceSpan,
}

impl ForbiddenLayerAccess {
    pub fn from_span_and_location(
        span: CodeSpan,
        location: &PathBuf,
        layer: String,
        layer_module: String,
        accessed_in: String,
    ) -> Self {
        let sample = fs::read_to_string(location).expect("path exists");
        let sample = get_code_sample_region(&sample, &span);
        // TODO: make that a function, call it once in the upper level
        let name = env!("CARGO_CRATE_NAME", "'CARGO_CRATE_NAME' should be set");
        let module_relative_path = layer_module
            .strip_prefix(&format!("{name}::"))
            .expect("module prefix ");
        let start_hint = sample.find(module_relative_path).expect("path to exist");

        let span = (start_hint, module_relative_path.len()).into();
        let base = std::env::current_dir().expect("path to exist");
        let location = location
            .strip_prefix(base)
            .expect("location to be in current dir");
        let src = NamedSource::new(location.to_string_lossy(), sample);
        ForbiddenLayerAccess {
            layer,
            layer_module,
            accessed_in,
            src,
            span,
        }
    }
}
