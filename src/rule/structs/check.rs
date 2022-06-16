use crate::ast::structs::Struct;
use crate::rule::assertable::Assertable;
use crate::rule::impl_block::impl_matches;
use crate::rule::structs::condition::struct_matches;
use crate::rule::structs::{
    AssertionConjunction, AssertionToken, ConditionToken, SimpleAssertions, StructMatches,
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
        self.assertion_result.push_expected("Structs that ");

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
                ConditionToken::Derives(trait_) => {
                    let expected = format!("derive {trait_}");
                    self.assertion_result.push_expected(&expected);
                    match_against.structs_that(|struct_| struct_.derives(&trait_))
                }
                ConditionToken::Implement(trait_) => {
                    let expected = format!("implement {trait_}");
                    self.assertion_result.push_expected(&expected);
                    let imps = impl_matches().impl_that(|imp| match &imp.trait_impl {
                        Some(t) if t.contains(&trait_) => true,
                        _ => false,
                    });
                    let types = imps.types();
                    println!("{:?}", types);
                    match_against.structs_that(|struct_| types.contains(&struct_.ident.as_str()))
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
                    SimpleAssertions::OnlyHavePrivateFields => self.assert_private_fields(),
                    SimpleAssertions::OnlyHavePublicFields => self.assert_public_fields(),
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

impl ArchRule<ConditionToken, AssertionToken, StructMatches> {
    fn assert_public(&mut self) -> bool {
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

    fn assert_private(&mut self) -> bool {
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

    fn assert_simple_name(&mut self, name: String) -> bool {
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

    fn assert_derives(&mut self, trait_: &String) -> bool {
        self.assertion_result
            .push_expected(format!("derive '{trait_}'"));

        let struct_without_expected_derive = self
            .subject
            .0
            .iter()
            .filter(|struct_| !struct_.derives.contains(trait_))
            .collect::<Vec<_>>();

        if !struct_without_expected_derive.is_empty() {
            self.assertion_result.push_actual(&format!(
                "the following structs does not derive '{trait_}':\n"
            ));
            struct_without_expected_derive.iter().for_each(|struct_| {
                self.assertion_result
                    .push_actual(format!("\t- {}\n", struct_.path))
            });

            false
        } else {
            true
        }
    }

    fn assert_implement(&mut self, trait_: &String) -> bool {
        self.assertion_result
            .push_expected(format!("implement '{trait_}'"));

        let struct_without_expected_impl = self
            .subject
            .0
            .iter()
            .filter(|struct_| {
                let imp_for_type =
                    impl_matches().impl_that(|imp| imp.self_ty.name() == struct_.ident.as_str());
                let imp_for_type = imp_for_type.impl_that(|imp| match &imp.trait_impl {
                    Some(t) if t.contains(trait_) => true,
                    _ => false,
                });
                imp_for_type.is_empty()
            })
            .collect::<Vec<_>>();

        if !struct_without_expected_impl.is_empty() {
            self.assertion_result.push_actual(&format!(
                "the following structs does not implement '{trait_}':\n"
            ));
            struct_without_expected_impl.iter().for_each(|struct_| {
                self.assertion_result
                    .push_actual(format!("\t- {}\n", struct_.path))
            });

            false
        } else {
            true
        }
    }

    fn assert_private_fields(&mut self) -> bool {
        self.assertion_result
            .push_expected("only have private fields");
        let struct_with_only_public_fields = self
            .subject
            .0
            .iter()
            .filter(|struct_| !struct_.fields.is_empty())
            .filter(|struct_| !struct_.has_non_public_field())
            .collect::<Vec<_>>();

        if !struct_with_only_public_fields.is_empty() {
            self.assertion_result
                .push_actual("the following structs have only public fields:\n");

            struct_with_only_public_fields.iter().for_each(|struct_| {
                self.assertion_result
                    .push_actual(format!("\t- {}\n", struct_.path))
            });

            false
        } else {
            true
        }
    }

    fn assert_public_fields(&mut self) -> bool {
        self.assertion_result
            .push_expected("only have public fields");
        let struct_with_non_public_fields = self
            .subject
            .0
            .iter()
            .filter(|struct_| !struct_.fields.is_empty())
            .filter(|struct_| struct_.has_non_public_field())
            .collect::<Vec<_>>();

        if !struct_with_non_public_fields.is_empty() {
            self.assertion_result
                .push_actual("the following structs have non public fields':\n");

            struct_with_non_public_fields.iter().for_each(|struct_| {
                self.assertion_result
                    .push_actual(format!("\t- {}\n", struct_.path))
            });

            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod condition_test {
    use crate::rule::structs::Structs;
    use crate::rule::{ArchRuleBuilder, CheckRule};

    #[test]
    #[should_panic]
    fn should_check_derives_panic() {
        Structs::that()
            .reside_in_a_module("modules")
            .should()
            .derive("Eq")
            .check();
    }

    #[test]
    fn should_check_derives_ok() {
        Structs::that()
            .reside_in_a_module("assertion_result")
            .should()
            .derive("Debug")
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_private_fields() {
        Structs::that()
            .are_declared_public()
            .should()
            .only_have_private_fields()
            .check();
    }

    #[test]
    fn should_check_public_fields_ok() {
        Structs::that()
            .reside_in_a_module("assertion_result")
            .should()
            .only_have_public_fields()
            .check();
    }

    #[test]
    #[should_panic]
    fn should_filter_implementors() {
        Structs::that()
            .implement("Display")
            .should()
            .be_private()
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_implementation() {
        Structs::that()
            .have_simple_name("AssertionResult")
            .should()
            .implement("Debug")
            .check();
    }
}
