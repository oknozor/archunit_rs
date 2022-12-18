use crate::assertion_result::AssertionResult;
use crate::ast::{module_tree, ItemPath, ModuleUse};
use crate::rule::modules::report::ModuleRuleViolation;
use crate::rule::modules::ModuleMatches;
use crate::rule::modules::{
    AssertionConjunction, AssertionToken, ConditionToken, DependencyAssertion,
    DependencyAssertionConjunction, ModulePredicateConjunctionBuilder, SimpleAssertions,
};
use crate::rule::pattern::PathPattern;
use crate::rule::{assertable::Assertable, ArchRule, CheckRule};
use crate::ModuleTree;
use std::collections::HashMap;

impl
    CheckRule<
        ConditionToken,
        AssertionToken,
        ModuleMatches,
        ArchRule<ConditionToken, AssertionToken, ModuleMatches>,
    > for ModulePredicateConjunctionBuilder
{
    fn get_rule(self) -> ArchRule<ConditionToken, AssertionToken, ModuleMatches> {
        self.0
    }
}

impl Assertable<ConditionToken, AssertionToken, ModuleMatches>
    for ArchRule<ConditionToken, AssertionToken, ModuleMatches>
{
    fn apply_conditions(&mut self) {
        let mut matches = ModuleMatches::default();
        let modules = module_tree().flatten(&self.filters);

        enum Conjunction {
            Or,
            And,
        }

        let mut conjunction = Conjunction::Or;
        self.assertion_results.push_expected("Modules that ");

        while let Some(condition) = self.conditions.pop_back() {
            let match_against = match conjunction {
                Conjunction::Or => &modules,
                Conjunction::And => &matches,
            };

            let matches_for_condition = match condition {
                ConditionToken::AreDeclaredPublic => {
                    self.assertion_results.push_expected("are declared public");
                    match_against
                        .0
                        .values()
                        .flat_map(|module| {
                            module.module_that(|sub| sub.is_public(), &self.filters).0
                        })
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::AreDeclaredPrivate => {
                    self.assertion_results.push_expected("are declared private");
                    match_against
                        .0
                        .values()
                        .flat_map(|module| {
                            module.module_that(|sub| !sub.is_public(), &self.filters).0
                        })
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::HaveSimpleName(name) => {
                    self.assertion_results
                        .push_expected(format!("have simple name '{name}'"));

                    match_against
                        .0
                        .values()
                        .flat_map(|module| {
                            module.module_that(|sub| sub.ident == name, &self.filters).0
                        })
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::HaveSimpleEndingWith(pattern) => {
                    self.assertion_results
                        .push_expected(format!("have simple name ending with '{pattern}'"));
                    match_against
                        .0
                        .values()
                        .flat_map(|module| {
                            module
                                .module_that(|sub| sub.ident.ends_with(&pattern), &self.filters)
                                .0
                        })
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::HaveSimpleStartingWith(pattern) => {
                    self.assertion_results
                        .push_expected(format!("have simple name starting with '{}'", &pattern));

                    match_against
                        .0
                        .values()
                        .flat_map(|module| {
                            module
                                .module_that(|sub| sub.ident.starts_with(&pattern), &self.filters)
                                .0
                        })
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::ResidesInAModule(name) => {
                    self.assertion_results
                        .push_expected(format!("resides in a modules that match '{name}'"));
                    match_against
                        .0
                        .values()
                        .flat_map(|module| {
                            module
                                .module_that(|sub| sub.path_match(&name), &self.filters)
                                .0
                        })
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::And => {
                    self.assertion_results.push_expected(" and ");
                    conjunction = Conjunction::And;
                    continue;
                }
                ConditionToken::Or => {
                    self.assertion_results.push_expected(" or ");
                    conjunction = Conjunction::Or;
                    continue;
                }
                ConditionToken::Should => {
                    self.assertion_results.push_expected(" to ");
                    break;
                }
            };

            match conjunction {
                Conjunction::Or => matches.extend(ModuleMatches(matches_for_condition)),
                Conjunction::And => matches = ModuleMatches(matches_for_condition),
            }
        }

        self.subject = matches
    }

    fn apply_assertions(&mut self) -> bool {
        enum Conjunction {
            Or,
            And,
        }

        let mut success = false;
        let mut conjunction = Conjunction::Or;

        while let Some(assertion) = self.assertions.pop_back() {
            let assertion_outcome = match assertion {
                AssertionToken::SimpleAssertion(assertion) => match assertion {
                    SimpleAssertions::BePublic => self.assert_public(),
                    SimpleAssertions::BePrivate => self.assert_private(),
                    SimpleAssertions::HaveSimpleName(name) => self.assert_simple_name(&name),
                },
                AssertionToken::Conjunction(a) => match a {
                    AssertionConjunction::AndShould => {
                        conjunction = Conjunction::And;
                        true
                    }
                    AssertionConjunction::OrShould => {
                        conjunction = Conjunction::Or;
                        true
                    }
                },
                AssertionToken::DependencyAssertion(dependency_assertion) => {
                    match dependency_assertion {
                        DependencyAssertion::That => true,
                    }
                }
                AssertionToken::DependencyAssertionConjunction(
                    dependency_assertion_conjunction,
                ) => match dependency_assertion_conjunction {
                    DependencyAssertionConjunction::OnlyHaveDependencyModule => {
                        // Skip mandatory `that` token
                        let _that = self.assertions.pop_back();
                        // While we don't have a conjunction we are in the dependency assertion
                        let mut assertion_result = true;
                        while let Some(token) = self.assertions.pop_back() {
                            match token {
                                AssertionToken::SimpleAssertion(
                                    SimpleAssertions::HaveSimpleName(name),
                                ) => {
                                    assertion_result = self.assert_dependencies_name_match(&name);
                                }
                                AssertionToken::Conjunction(AssertionConjunction::AndShould) => {
                                    self.assertion_results.push_expected(" and ");
                                    conjunction = Conjunction::And;
                                    break;
                                }
                                AssertionToken::Conjunction(AssertionConjunction::OrShould) => {
                                    self.assertion_results.push_expected(" or ");
                                    conjunction = Conjunction::Or;
                                    break;
                                }
                                other => panic!(
                                    "Unsupported nested module dependency assertion {other:?}"
                                ),
                            }
                        }

                        assertion_result
                    }
                },
            };

            match conjunction {
                Conjunction::Or => success = success || assertion_outcome,
                Conjunction::And => success = success && assertion_outcome,
            };
        }

        success
    }

    fn assertion_results(&self) -> &AssertionResult {
        &self.assertion_results
    }
}

impl ArchRule<ConditionToken, AssertionToken, ModuleMatches> {
    fn assert_public(&mut self) -> bool {
        self.assertion_results.push_expected("be public");
        let non_public_modules = self
            .subject
            .0
            .values()
            .filter(|module| !module.is_public())
            .collect::<Vec<_>>();
        for module in &non_public_modules {
            let declaration = module.declaration.as_ref().expect("should be declared");
            self.assertion_results
                .push_actual(ModuleRuleViolation::be_public(
                    declaration.span,
                    &declaration.real_path,
                    declaration.ident.clone(),
                    declaration.vis,
                ))
        }
        non_public_modules.is_empty()
    }

    fn assert_private(&mut self) -> bool {
        self.assertion_results.push_expected("be private");
        let public_modules = self
            .subject
            .0
            .values()
            .filter(|module| module.is_public())
            .collect::<Vec<_>>();

        for module in &public_modules {
            self.assertion_results
                .push_actual(ModuleRuleViolation::be_private(
                    module
                        .span
                        .expect("Should not try to get span for crate root"),
                    &module.real_path,
                    module.ident.clone(),
                    module.visibility,
                ))
        }

        public_modules.is_empty()
    }

    fn assert_simple_name(&mut self, name: &str) -> bool {
        self.assertion_results
            .push_expected(format!("have simple name '{name}'"));
        let module_with_non_matching_name = self
            .subject
            .0
            .values()
            .filter(|module| module.ident != name)
            .collect::<Vec<_>>();

        for module in &module_with_non_matching_name {
            let declaration = module
                .declaration
                .as_ref()
                .expect("module should have declaration");
            self.assertion_results
                .push_actual(ModuleRuleViolation::have_name_matching(
                    declaration.span,
                    name.to_owned(),
                    &declaration.real_path,
                    declaration.ident.clone(),
                ))
        }

        module_with_non_matching_name.is_empty()
    }

    fn assert_dependencies_name_match(&mut self, pattern: &str) -> bool {
        self.assertion_results.push_expected(format!(
            "only have dependencies matching pattern '{pattern}'"
        ));
        let dependencies_matches = self
            .subject
            .0
            .values()
            .map(|module| module.flatten_deps(&self.filters))
            .collect::<Vec<_>>();

        let mut per_module_mismatch = vec![];
        for dependency_match in dependencies_matches {
            for (path, (real_path, deps)) in dependency_match.0 {
                let mismatch_deps: Vec<&ModuleUse> =
                    deps.iter().filter(|dep| !dep.matching(pattern)).collect();

                if !mismatch_deps.is_empty() {
                    per_module_mismatch.push((path, real_path, mismatch_deps));
                }
            }
        }

        if per_module_mismatch.is_empty() {
            true
        } else {
            for (path, real_path, usage) in per_module_mismatch {
                usage.into_iter().for_each(|usage| {
                    self.assertion_results.push_actual(
                        ModuleRuleViolation::only_have_dependencies_with_simple_name(
                            usage.span,
                            real_path,
                            path.to_string(),
                            pattern.to_owned(),
                            usage.parts.to_owned(),
                        ),
                    )
                });
            }
            false
        }
    }
}

impl ModuleUse {
    pub fn matching(&self, pattern: &str) -> bool {
        PathPattern::from(pattern).matches_module_path(&self.parts)
    }

    pub fn starts_with(&self, path: &str) -> bool {
        if self.parts.starts_with("crate") {
            let name = env!("CARGO_CRATE_NAME", "'CARGO_CRATE_NAME' should be set");
            let relative_path = &self.parts[5..];
            let parts_canonical = &format!("{name}{relative_path}");
            let pattern = format!("{path}*");
            PathPattern::from(pattern.as_str()).matches_module_path(parts_canonical)
        } else {
            let pattern1 = format!("{path}*");
            PathPattern::from(pattern1.as_str()).matches_module_path(&self.parts)
        }
    }
}

#[cfg(test)]
mod condition_test {
    use crate::ast::{CodeSpan, ModuleUse};
    use crate::rule::modules::Modules;
    use crate::rule::{ArchRuleBuilder, CheckRule};
    use crate::Filters;
    use speculoos::prelude::*;

    #[test]
    fn should_match_module_use_start() {
        let module_usage = ModuleUse {
            parts: "archunit_rs::rule::enums::Enums".to_owned(),
            span: CodeSpan::default(),
        };

        assert_that!(module_usage.starts_with("archunit_rs::rule")).is_true();
        assert_that!(module_usage.starts_with("archunit_rs::ast")).is_false();
        assert_that!(module_usage.starts_with("ast")).is_false();
    }

    #[test]
    fn should_match_module_use_start_start_when_usage_start_with_crate() {
        let module_usage = ModuleUse {
            parts: "crate::rule::enums::Enums".to_owned(),
            span: CodeSpan::default(),
        };

        assert_that!(module_usage.starts_with("archunit_rs::rule")).is_true();
        assert_that!(module_usage.starts_with("archunit_rs::ast")).is_false();
        assert_that!(module_usage.starts_with("ast")).is_false();
    }

    #[test]
    #[should_panic]
    fn module_should_have_simple_name_panics() {
        let excluding_test = Filters::default().exclude_test();

        Modules::that(excluding_test)
            .reside_in_a_module("archunit_rs::rule::modules::*")
            .should()
            .have_simple_name("report")
            .check();
    }

    #[test]
    fn module_should_have_simple_name_ok() {
        let excluding_test = Filters::default().exclude_test();

        Modules::that(excluding_test)
            .reside_in_a_module("archunit_rs::rule::modules::*")
            .should()
            .have_simple_name("report")
            .or_should()
            .have_simple_name("condition")
            .or_should()
            .have_simple_name("check")
            .check();
    }

    #[test]
    #[should_panic]
    fn module_should_be_public_panics() {
        Modules::that(Filters::default().exclude_test())
            .reside_in_a_module("archunit_rs::rule::modules::*")
            .or()
            .have_simple_name("ast")
            .should()
            .be_public()
            .check();
    }

    #[test]
    fn module_should_be_private_ok_excluding_cfg_test() {
        let excluding_test = Filters::default().exclude_test();

        Modules::that(excluding_test)
            .reside_in_a_module("archunit_rs::rule::modules::*")
            .or()
            .have_simple_name("ast")
            .should()
            .be_private()
            .check();
    }

    #[test]
    #[should_panic]
    fn should_panic_when_dependencies_does_not_match_pattern() {
        Modules::that(Filters::default())
            .have_simple_name("pattern")
            .should()
            .only_have_dependency_module()
            .that()
            .have_simple_name("wildmatch")
            .check()
    }

    #[test]
    fn should_not_panic_when_dependencies_matches_pattern() {
        let excluding_test = Filters::default().exclude_test();

        Modules::that(excluding_test)
            .have_simple_name("pattern")
            .should()
            .only_have_dependency_module()
            .that()
            .have_simple_name("wildmatch")
            .check()
    }
}
