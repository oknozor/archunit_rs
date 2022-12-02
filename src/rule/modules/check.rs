use crate::ast::{module_tree, ItemPath, ModuleUse};
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
        let modules = module_tree().flatten();

        enum Conjunction {
            Or,
            And,
        }

        let mut conjunction = Conjunction::Or;
        self.assertion_result.push_expected("Modules that ");

        while let Some(condition) = self.conditions.pop_back() {
            let match_against = match conjunction {
                Conjunction::Or => &modules,
                Conjunction::And => &matches,
            };

            let matches_for_condition = match condition {
                ConditionToken::AreDeclaredPublic => {
                    self.assertion_result.push_expected("are declared public");
                    match_against
                        .0
                        .values()
                        .flat_map(|module| module.module_that(|sub| sub.is_public()).0)
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::AreDeclaredPrivate => {
                    self.assertion_result.push_expected("are declared private");
                    match_against
                        .0
                        .values()
                        .flat_map(|module| module.module_that(|sub| !sub.is_public()).0)
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::HaveSimpleName(name) => {
                    self.assertion_result
                        .push_expected(format!("have simple name '{}'", name));

                    match_against
                        .0
                        .values()
                        .flat_map(|module| module.module_that(|sub| sub.ident == name).0)
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::HaveSimpleEndingWith(pattern) => {
                    self.assertion_result
                        .push_expected(format!("have simple name ending with '{}'", pattern));
                    match_against
                        .0
                        .values()
                        .flat_map(|module| {
                            module.module_that(|sub| sub.ident.ends_with(&pattern)).0
                        })
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::HaveSimpleStartingWith(pattern) => {
                    self.assertion_result
                        .push_expected(format!("have simple name starting with '{}'", &pattern));

                    match_against
                        .0
                        .values()
                        .flat_map(|module| {
                            module.module_that(|sub| sub.ident.starts_with(&pattern)).0
                        })
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::ResidesInAModule(name) => {
                    self.assertion_result
                        .push_expected(format!("resides in a modules that match '{}'", name));
                    match_against
                        .0
                        .values()
                        .flat_map(|module| module.module_that(|sub| sub.path_match(&name)).0)
                        .collect::<HashMap<&ItemPath, &ModuleTree>>()
                }
                ConditionToken::And => {
                    self.assertion_result.push_expected(" and ");
                    conjunction = Conjunction::And;
                    continue;
                }
                ConditionToken::Or => {
                    self.assertion_result.push_expected(" or ");
                    conjunction = Conjunction::Or;
                    continue;
                }
                ConditionToken::Should => {
                    self.assertion_result.push_expected(" to ");
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

    fn apply_assertions(&mut self) {
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
                    SimpleAssertions::HaveSimpleName(name) => self.assert_simple_name(name),
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
                                    self.assertion_result.push_expected(" and ");
                                    conjunction = Conjunction::And;
                                    break;
                                }
                                AssertionToken::Conjunction(AssertionConjunction::OrShould) => {
                                    self.assertion_result.push_expected(" or ");
                                    conjunction = Conjunction::Or;
                                    break;
                                }
                                other => unimplemented!("Unexpected assertion token {other:?}"),
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

        if !success {
            panic!("\n{}", self.assertion_result)
        }
    }
}

impl ArchRule<ConditionToken, AssertionToken, ModuleMatches> {
    fn assert_public(&mut self) -> bool {
        self.assertion_result.push_expected("be public");
        let non_public_modules = self
            .subject
            .0
            .values()
            .filter(|module| !module.is_public())
            .collect::<Vec<_>>();

        if !non_public_modules.is_empty() {
            self.assertion_result
                .push_actual("the following modules are not public:\n");
            non_public_modules.iter().for_each(|module| {
                self.assertion_result.push_actual(format!(
                    "\t{} - visibility : {:?}\n",
                    module.path, module.visibility
                ))
            });

            false
        } else {
            true
        }
    }

    fn assert_private(&mut self) -> bool {
        self.assertion_result.push_expected("be private");
        let public_modules = self
            .subject
            .0
            .values()
            .filter(|module| module.is_public())
            .collect::<Vec<_>>();

        if !public_modules.is_empty() {
            self.assertion_result.push_actual("Found public module:\n");
            public_modules.iter().for_each(|module| {
                self.assertion_result.push_actual(format!(
                    "\t{} - visibility : {:?}\n",
                    module.path, module.visibility
                ))
            });

            false
        } else {
            true
        }
    }

    fn assert_simple_name(&mut self, name: String) -> bool {
        self.assertion_result
            .push_expected(format!("have simple name '{}'", name));
        let module_with_non_matching_name = self
            .subject
            .0
            .values()
            .filter(|module| module.ident != name)
            .collect::<Vec<_>>();

        if !module_with_non_matching_name.is_empty() {
            self.assertion_result
                .push_actual("the following modules have a different name:\n");
            module_with_non_matching_name.iter().for_each(|module| {
                self.assertion_result
                    .push_actual(format!("{}\n", module.path))
            });

            false
        } else {
            true
        }
    }

    fn assert_dependencies_name_match(&mut self, pattern: &str) -> bool {
        self.assertion_result.push_expected(format!(
            "only have dependencies matching pattern '{pattern}'"
        ));
        let dependencies_matches = self
            .subject
            .0
            .values()
            .map(|module| module.flatten_deps())
            .collect::<Vec<_>>();

        let mut per_module_mismatch = vec![];
        for dependency_match in dependencies_matches {
            for (path, deps) in dependency_match.0 {
                println!("{:?}", (path, deps));
                let mismatch_deps: Vec<&ModuleUse> =
                    deps.iter().filter(|dep| !dep.matching(pattern)).collect();

                if !mismatch_deps.is_empty() {
                    per_module_mismatch.push((path, mismatch_deps));
                }
            }
        }

        if per_module_mismatch.is_empty() {
            true
        } else {
            for (path, usage) in per_module_mismatch {
                self.assertion_result.push_actual(format!(
                    "\n\nDependencies in '{path}' don't match '{pattern}':"
                ));
                for usage in usage {
                    self.assertion_result
                        .push_actual(format!("\n\t- 'use {}'", usage.parts))
                }
                self.assertion_result.push_actual("\n");
            }
            self.assertion_result.push_actual("\n");
            false
        }
    }
}

impl ModuleUse {
    pub fn matching(&self, pattern: &str) -> bool {
        PathPattern::from(pattern).matches_module_path(&self.parts)
    }
}

#[cfg(test)]
mod condition_test {
    use crate::rule::modules::Modules;
    use crate::rule::{ArchRuleBuilder, CheckRule};

    #[test]
    #[should_panic]
    fn should_check_assertion() {
        Modules::that()
            .reside_in_a_module("archunit_rs::rule::modules::*")
            .or()
            .have_simple_name("ast")
            .should()
            .be_public()
            .check();
    }

    #[test]
    fn should_check_dependency_assertions() {
        Modules::that()
            .have_simple_name("pattern")
            .should()
            .only_have_dependency_module()
            .that()
            .have_simple_name("wildmatch")
            .and_should()
            .be_private()
            .check()
    }
}
