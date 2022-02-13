use std::collections::HashMap;

use crate::ast::{ItemPath, ModuleTree};

#[derive(Default)]
pub struct ModuleMatches(pub HashMap<&'static ItemPath, &'static ModuleTree>);

impl ModuleMatches {
    pub fn extend(&mut self, other: ModuleMatches) {
        self.0.extend(other.0);
    }
}
impl ModuleTree {
    pub(crate) fn module_that<P>(&'static self, mut predicate: P) -> ModuleMatches
    where
        P: FnMut(&&ModuleTree) -> bool,
    {
        let matches = self
            .flatten()
            .0
            .into_iter()
            .filter(|(_, module)| predicate(module))
            .collect::<HashMap<&ItemPath, &ModuleTree>>();

        ModuleMatches(matches)
    }

    pub(crate) fn flatten(&'static self) -> ModuleMatches {
        let mut modules = HashMap::new();
        modules.insert(&self.path, self);

        self.submodules
            .iter()
            .flat_map(|sub| sub.flatten().0)
            .for_each(|(path, module)| {
                modules.insert(path, module);
            });

        ModuleMatches(modules)
    }
}

#[cfg(test)]
mod condition_test {
    use crate::rule::assertable::Assertable;
    use speculoos::prelude::*;

    use crate::rule::modules::Modules;
    use crate::rule::ArchRuleBuilder;

    #[test]
    fn should_filter_modules_with_and_conjunctions() {
        let mut arch_rule = Modules::that()
            .reside_in_a_module("modules")
            .and()
            .have_simple_name("condition");

        arch_rule.0.apply_conditions();

        let paths = arch_rule
            .0
            .subject
            .0
            .keys()
            .map(|key| key.as_str())
            .collect::<Vec<&str>>();

        assert_that!(arch_rule.0.assertion_result.expected).is_equal_to(
            &"Modules that resides in a modules named 'modules' and have simple name 'condition'"
                .to_string(),
        );

        assert_that!(paths).contains_all_of(&[&"archunit_rs::rule::modules::condition"]);
    }

    #[test]
    fn should_filter_modules_with_or_conjunctions() {
        let mut arch_rule = Modules::that()
            .reside_in_a_module("modules")
            .or()
            .have_simple_name("ast")
            .0;

        arch_rule.apply_conditions();

        let paths = arch_rule
            .subject
            .0
            .keys()
            .map(|key| key.as_str())
            .collect::<Vec<&str>>();

        assert_that!(arch_rule.assertion_result.expected).is_equal_to(
            &"Modules that resides in a modules named 'modules' or have simple name 'ast'"
                .to_string(),
        );

        assert_that!(paths).contains_all_of(&[
            &"archunit_rs::rule::modules::module_test",
            &"archunit_rs::rule::modules::condition",
            &"archunit_rs::rule::modules::condition::condition_test",
            &"archunit_rs::rule::modules::check",
            &"archunit_rs::ast",
        ]);
    }
}
