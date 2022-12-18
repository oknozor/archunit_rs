#![warn(
    clippy::todo,
    clippy::string_to_string,
    clippy::str_to_string,
    clippy::unneeded_field_pattern,
    clippy::unwrap_used
)]

pub mod assertion_result;
mod ast;
pub mod layer_rule;
pub mod rule;

use ast::ModuleTree;
pub use rule::modules::Modules;
pub use rule::structs::Structs;

/// Control what to filters when running Archunit tests
#[derive(Default, Debug, Clone)]
pub struct ExludeModules<'a> {
    pub(crate) exclude_cfg: Vec<&'a str>,
}

impl<'a> ExludeModules<'a> {
    pub fn cfg_test() -> Self {
        Self {
            exclude_cfg: vec!["test"],
        }
    }
    /// Excludes all modules with the given cfg sattribute
    pub fn exclude_cfg(mut self, cfg_attr: &'static str) -> Self {
        self.exclude_cfg.push(cfg_attr);
        self
    }

    fn filter(&self) -> impl FnMut(&&ModuleTree) -> bool + '_ {
        move |module: &&ModuleTree| {
            !module
                .cfg_attr
                .iter()
                .any(|attr| self.exclude_cfg.contains(&attr.as_str()))
        }
    }
}

#[cfg(test)]
mod thread_local_filter_test {
    use crate::rule::{ArchRuleBuilder, CheckRule};
    use crate::{ExludeModules, Structs};

    // A struct that makes the arch test below fail (being private)
    #[derive(Debug)]
    struct RuleViolation;

    #[test]
    fn test_should_filter_manually() {
        Structs::that(ExludeModules::cfg_test())
            .reside_in_a_module("archunit_rs::thread_local_filter_test")
            .should()
            .be_public()
            .check();
    }

    #[test]
    #[should_panic]
    fn should_not_filter() {
        Structs::that(ExludeModules::default())
            .reside_in_a_module("archunit_rs::thread_local_filter_test")
            .should()
            .be_public()
            .check();
    }
}
