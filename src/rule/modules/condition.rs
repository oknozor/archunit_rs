use std::collections::HashMap;

use crate::ast::{ItemPath, ModuleTree};
use crate::rule::modules::{ModuleDependencies, ModuleMatches};

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

    pub(crate) fn flatten_deps(&'static self) -> ModuleDependencies {
        let mut modules = HashMap::new();
        modules.insert(&self.path, (&self.real_path, &self.dependencies));

        self.submodules
            .iter()
            .flat_map(|sub| sub.flatten().0)
            .map(|(p, m)| (p, &m.real_path, &m.dependencies))
            .for_each(|(path, real_path, deps)| {
                modules.insert(path, (real_path, deps));
            });

        ModuleDependencies(modules)
    }
}

#[cfg(test)]
mod condition_test {
    use crate::ast::module_tree;
    use crate::rule::assertable::Assertable;
    use speculoos::prelude::*;

    use crate::rule::modules::Modules;
    use crate::rule::ArchRuleBuilder;

    #[test]
    fn filter_out_a_module_and_its_children() {
        let matches =
            module_tree().module_that(|module| !module.path.reside_in("archunit_rs::rule"));
        let matches: Vec<String> = matches.0.keys().map(|path| path.to_string()).collect();

        for path in matches {
            assert_that!(path.starts_with("archunit_rs::rule")).is_false();
        }
    }

    #[test]
    fn keep_only_a_module_and_its_children() {
        let matches =
            module_tree().module_that(|module| module.path.reside_in("archunit_rs::rule"));
        let matches: Vec<String> = matches.0.keys().map(|path| path.to_string()).collect();

        for path in matches {
            assert_that!(path).starts_with("archunit_rs::rule");
        }
    }

    #[test]
    fn should_filter_modules_with_and_conjunctions() {
        let mut arch_rule = Modules::that()
            .reside_in_a_module("*::modules")
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

        assert_that!(arch_rule.0.assertion_results.expected).is_equal_to(
            &"Modules that resides in a modules that match '*::modules' and have simple name 'condition'"
                .to_string(),
        );

        assert_that!(paths).contains_all_of(&[&"archunit_rs::rule::modules::condition"]);
    }

    #[test]
    fn should_filter_modules_with_or_conjunctions() {
        let mut arch_rule = Modules::that()
            .reside_in_a_module("archunit_rs::rule::modules::*")
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

        assert_that!(arch_rule.assertion_results.expected).is_equal_to(
            &"Modules that resides in a modules that match 'archunit_rs::rule::modules::*' or have simple name 'ast'"
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
