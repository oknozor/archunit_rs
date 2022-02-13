use crate::ast::Struct;
use crate::rule::assertable::Assertable;
use crate::rule::structs::condition::{struct_matches, StructMatches};
use crate::rule::structs::{
    AssertionConjunction, AssertionToken, ConditionToken, SimpleAssertions,
    StructPredicateConjunctionBuilder,
};
use crate::rule::{ArchRule, CheckRule};

impl
    CheckRule<
        ConditionToken,
        AssertionToken,
        StructMatches,
        ArchRule<ConditionToken, AssertionToken, StructMatches>,
    > for StructPredicateConjunctionBuilder
{
    fn get_rule(self) -> ArchRule<ConditionToken, AssertionToken, StructMatches> {
        self.0
    }
}

impl Assertable<ConditionToken, AssertionToken, StructMatches>
    for ArchRule<ConditionToken, AssertionToken, StructMatches>
{
    fn apply_conditions(&mut self) {
        let mut matches = StructMatches::default();
        let structs = struct_matches();

        enum Conjunction {
            Or,
            And,
        }

        let mut conjunction = Conjunction::Or;
        self.assertion_result.push_expected("Modules that ");

        while let Some(condition) = self.conditions.pop_back() {
            let match_against = match conjunction {
                Conjunction::Or => structs,
                Conjunction::And => &matches,
            };

            let matches_for_condition = match condition {
                ConditionToken::AreDeclaredPublic => {
                    self.assertion_result.push_expected("are declared public");
                    match_against.structs_that(Struct::is_public)
                }
                ConditionToken::AreDeclaredPrivate => {
                    self.assertion_result.push_expected("are declared private");
                    match_against.structs_that(|struct_| !struct_.is_public())
                }
                ConditionToken::HaveSimpleName(name) => {
                    self.assertion_result
                        .push_expected(format!("have simple name '{}'", name));
                    match_against.structs_that(|struct_| struct_.ident == name)
                }
                ConditionToken::ResidesInAModule(name) => {
                    self.assertion_result
                        .push_expected(format!("resides in a modules named '{}'", name));
                    match_against.structs_that(|struct_| struct_.has_parent(&name))
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
                ConditionToken::Derives(_) => {
                    todo!()
                }
                ConditionToken::Implement(_) => {
                    todo!()
                }
            };

            match conjunction {
                Conjunction::Or => matches.extends(matches_for_condition),
                Conjunction::And => matches = matches_for_condition,
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
                        let non_public_struct = self
                            .subject
                            .0
                            .iter()
                            .filter(|struct_| !struct_.is_public())
                            .collect::<Vec<_>>();

                        if !non_public_struct.is_empty() {
                            self.assertion_result
                                .push_actual("the following structs are not public:\n");
                            non_public_struct.iter().for_each(|struct_| {
                                self.assertion_result.push_actual(format!(
                                    "\t{} - visibility : {:?}\n",
                                    struct_.path, struct_.visibility
                                ))
                            });
                            false
                        } else {
                            true
                        }
                    }
                    SimpleAssertions::BePrivate => {
                        self.assertion_result.push_expected("be private");
                        let public_structs = self
                            .subject
                            .0
                            .iter()
                            .filter(|struct_| struct_.is_public())
                            .collect::<Vec<_>>();

                        if !public_structs.is_empty() {
                            self.assertion_result
                                .push_actual("the following structs are public:\n");
                            public_structs.iter().for_each(|struct_| {
                                self.assertion_result.push_actual(format!(
                                    "\t{} - visibility : {:?}\n",
                                    struct_.path, struct_.visibility
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
                        let struct_with_non_matching_name = self
                            .subject
                            .0
                            .iter()
                            .filter(|struct_| struct_.ident != name)
                            .collect::<Vec<_>>();

                        if !struct_with_non_matching_name.is_empty() {
                            self.assertion_result
                                .push_actual("the following structs have a different name:\n");
                            struct_with_non_matching_name.iter().for_each(|struct_| {
                                self.assertion_result
                                    .push_actual(format!("{}\n", struct_.path))
                            });

                            false
                        } else {
                            true
                        }
                    }
                    SimpleAssertions::Implement(_) => {
                        todo!()
                    }
                    SimpleAssertions::Derive(_) => {
                        todo!()
                    }
                    SimpleAssertions::OnlyHavePrivateFields => {
                        todo!()
                    }
                    SimpleAssertions::OnlyHavePublicFields => {
                        todo!()
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
