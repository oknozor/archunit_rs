use crate::ast::enums::Enum;
use crate::rule::assertable::Assertable;
use crate::rule::enums::{
    AssertionConjunction, AssertionToken, ConditionToken, EnumMatches,
    EnumPredicateConjunctionBuilder, SimpleAssertions,
};
use crate::rule::impl_block::impl_matches;
use crate::rule::structs::condition::enum_matches;
use crate::rule::{ArchRule, CheckRule};

impl
    CheckRule<
        ConditionToken,
        AssertionToken,
        EnumMatches,
        ArchRule<ConditionToken, AssertionToken, EnumMatches>,
    > for EnumPredicateConjunctionBuilder
{
    fn get_rule(self) -> ArchRule<ConditionToken, AssertionToken, EnumMatches> {
        self.0
    }
}

impl Assertable<ConditionToken, AssertionToken, EnumMatches>
    for ArchRule<ConditionToken, AssertionToken, EnumMatches>
{
    fn apply_conditions(&mut self) {
        let mut matches = EnumMatches::default();
        let enums = enum_matches();

        enum Conjunction {
            Or,
            And,
        }

        let mut conjunction = Conjunction::Or;
        self.assertion_result.push_expected("Structs that ");

        while let Some(condition) = self.conditions.pop_back() {
            let match_against = match conjunction {
                Conjunction::Or => enums,
                Conjunction::And => &matches,
            };

            let matches_for_condition = match condition {
                ConditionToken::AreDeclaredPublic => {
                    self.assertion_result.push_expected("are declared public");
                    match_against.enums_that(Enum::is_public)
                }
                ConditionToken::AreDeclaredPrivate => {
                    self.assertion_result.push_expected("are declared private");
                    match_against.enums_that(|enum_| !enum_.is_public())
                }
                ConditionToken::HaveSimpleName(name) => {
                    self.assertion_result
                        .push_expected(format!("have simple name '{}'", name));
                    match_against.enums_that(|enum_| enum_.ident == name)
                }
                ConditionToken::ResidesInAModule(name) => {
                    self.assertion_result
                        .push_expected(format!("resides in a modules that match '{}'", name));
                    match_against.enums_that(|enum_| enum_.path_match(&name))
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
                ConditionToken::Derives(trait_) => {
                    let expected = format!("derive {trait_}");
                    self.assertion_result.push_expected(&expected);
                    match_against.enums_that(|enum_| enum_.derives(&trait_))
                }
                ConditionToken::Implement(trait_) => {
                    let expected = format!("implement {trait_}");
                    self.assertion_result.push_expected(&expected);
                    let imps = impl_matches()
                        .impl_that(|imp| matches!(&imp.trait_impl, Some(t) if t.contains(&trait_)));
                    let types = imps.types();
                    match_against.enums_that(|enum_| types.contains(&enum_.ident.as_str()))
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
                    SimpleAssertions::BePublic => self.assert_public(),
                    SimpleAssertions::BePrivate => self.assert_private(),
                    SimpleAssertions::HaveSimpleName(name) => self.assert_simple_name(name),
                    SimpleAssertions::Implement(trait_) => self.assert_implement(&trait_),
                    SimpleAssertions::Derive(trait_) => self.assert_derives(&trait_),
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

impl ArchRule<ConditionToken, AssertionToken, EnumMatches> {
    fn assert_public(&mut self) -> bool {
        self.assertion_result.push_expected("be public");
        let non_public_struct = self
            .subject
            .0
            .iter()
            .filter(|enum_| !enum_.is_public())
            .collect::<Vec<_>>();

        if !non_public_struct.is_empty() {
            self.assertion_result
                .push_actual("the following enums are not public:\n");
            non_public_struct.iter().for_each(|enum_| {
                self.assertion_result.push_actual(format!(
                    "\t{} - visibility : {:?}\n",
                    enum_.path, enum_.visibility
                ))
            });
            false
        } else {
            true
        }
    }

    fn assert_private(&mut self) -> bool {
        self.assertion_result.push_expected("be private");
        let public_structs = self
            .subject
            .0
            .iter()
            .filter(|enum_| enum_.is_public())
            .collect::<Vec<_>>();

        if !public_structs.is_empty() {
            self.assertion_result
                .push_actual("the following enums are public:\n");
            public_structs.iter().for_each(|enum_| {
                self.assertion_result.push_actual(format!(
                    "\t{} - visibility : {:?}\n",
                    enum_.path, enum_.visibility
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
        let enum_with_non_matching_name = self
            .subject
            .0
            .iter()
            .filter(|enum_| enum_.ident != name)
            .collect::<Vec<_>>();

        if !enum_with_non_matching_name.is_empty() {
            self.assertion_result
                .push_actual("the following enums have a different name:\n");
            enum_with_non_matching_name.iter().for_each(|enum_| {
                self.assertion_result
                    .push_actual(format!("{}\n", enum_.path))
            });

            false
        } else {
            true
        }
    }

    fn assert_derives(&mut self, trait_: &String) -> bool {
        self.assertion_result
            .push_expected(format!("derive '{trait_}'"));

        let enum_without_expected_derive = self
            .subject
            .0
            .iter()
            .filter(|enum_| !enum_.derives.contains(trait_))
            .collect::<Vec<_>>();

        if !enum_without_expected_derive.is_empty() {
            self.assertion_result.push_actual(&format!(
                "the following enums does not derive '{trait_}':\n"
            ));
            enum_without_expected_derive.iter().for_each(|enum_| {
                self.assertion_result
                    .push_actual(format!("\t- {}\n", enum_.path))
            });

            false
        } else {
            true
        }
    }

    fn assert_implement(&mut self, trait_: &String) -> bool {
        self.assertion_result
            .push_expected(format!("implement '{trait_}'"));

        let enum_without_expected_impl = self
            .subject
            .0
            .iter()
            .filter(|enum_| {
                let imp_for_type =
                    impl_matches().impl_that(|imp| imp.self_ty.name() == enum_.ident.as_str());
                let imp_for_type = imp_for_type
                    .impl_that(|imp| matches!(&imp.trait_impl, Some(t) if t.contains(trait_)));
                imp_for_type.is_empty()
            })
            .collect::<Vec<_>>();

        if !enum_without_expected_impl.is_empty() {
            self.assertion_result.push_actual(&format!(
                "the following enums does not implement '{trait_}':\n"
            ));
            enum_without_expected_impl.iter().for_each(|enum_| {
                self.assertion_result
                    .push_actual(format!("\t- {}\n", enum_.path))
            });

            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod condition_test {
    use crate::rule::enums::Enums;
    use crate::rule::{ArchRuleBuilder, CheckRule};

    #[test]
    #[should_panic]
    fn should_check_derives_panic() {
        Enums::that()
            .reside_in_a_module("*::modules")
            .should()
            .derive("Deserialize")
            .check();
    }

    #[test]
    fn should_check_derives_ok() {
        Enums::that()
            .have_simple_name("Visibility")
            .should()
            .derive("Debug")
            .check();
    }

    #[test]
    fn should_filter_implementors() {
        Enums::that()
            .implement("Condition")
            .should()
            .have_simple_name("ConditionToken")
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_implementation() {
        Enums::that()
            .have_simple_name("AssertionToken")
            .should()
            .implement("Debug")
            .check();
    }
}
