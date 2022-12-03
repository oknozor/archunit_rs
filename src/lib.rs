#![warn(
    clippy::todo,
    clippy::string_to_string,
    clippy::str_to_string,
    clippy::unneeded_field_pattern,
    clippy::unwrap_used
)]
pub mod assertion_result;
pub mod ast;
pub mod layer_rule;
pub mod rule;
use ast::ModuleTree;

pub use rule::modules::Modules;
pub use rule::structs::Structs;

#[derive(Default)]
pub struct ModuleFilters;

impl ModuleFilters {
    pub fn exclude_test(self) -> Self {
        ast::visitor::push_cfg_filter("test");
        self
    }

    pub fn exclude_cfg(self, cfg_attr: &'static str) -> Self {
        ast::visitor::push_cfg_filter(cfg_attr);
        self
    }
}
