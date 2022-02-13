use crate::ast::{module_tree, ItemPath};
use crate::rule::modules::condition::ModuleMatches;
use crate::rule::modules::{
    AssertionConjunction, AssertionToken, ConditionToken, DependencyAssertion,
    DependencyAssertionConjunction, ModulePredicateConjunctionBuilder, SimpleAssertions,
};
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
                ConditionToken::ResidesInAModule(name) => {
                    self.assertion_result
                        .push_expected(format!("resides in a modules named '{}'", name));
                    match_against
                        .0
                        .values()
                        .flat_map(|module| module.module_that(|sub| sub.has_parent(&name)).0)
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
                    SimpleAssertions::BePublic => {
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
                    SimpleAssertions::BePrivate => {
                        self.assertion_result.push_expected("be private");
                        let public_modules = self
                            .subject
                            .0
                            .values()
                            .filter(|module| module.is_public())
                            .collect::<Vec<_>>();

                        if !public_modules.is_empty() {
                            self.assertion_result
                                .push_actual("the following modules are public:\n");
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
                    SimpleAssertions::HaveSimpleName(name) => {
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
                        todo!()
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

#[cfg(test)]
mod condition_test {
    use crate::rule::modules::Modules;
    use crate::rule::{ArchRuleBuilder, CheckRule};

    #[test]
    #[should_panic]
    fn should_check_assertion() {
        Modules::that()
            .reside_in_a_module("modules")
            .or()
            .have_simple_name("ast")
            .should()
            .be_public()
            .check();
    }
}
