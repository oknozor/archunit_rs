use crate::assertion_result::AssertionResult;
use crate::ast::structs::Struct;
use crate::rule::assertable::Assertable;
use crate::rule::impl_block::impl_matches;
use crate::rule::structs::condition::struct_matches;
use crate::rule::structs::reports::StructRuleViolation;
use crate::rule::structs::{
    AssertionConjunction, AssertionToken, ConditionToken, SimpleAssertions, StructMatches,
    StructPredicateConjunctionBuilder,
};
use crate::rule::{ArchRule, CheckRule};
use std::collections::HashSet;
use wildmatch::WildMatch;

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
        let structs = struct_matches(&self.filters);

        if self.conditions.is_empty() {
            self.assertion_results.push_expected("All structs should ");
            self.subject = structs.structs_that(Struct::all);
            return;
        };

        let mut matches = StructMatches::default();

        enum Conjunction {
            Or,
            And,
        }

        let mut conjunction = Conjunction::Or;
        self.assertion_results.push_expected("Structs that ");

        while let Some(condition) = self.conditions.pop_back() {
            let match_against = match conjunction {
                Conjunction::Or => &structs,
                Conjunction::And => &matches,
            };

            let matches_for_condition = match condition {
                ConditionToken::AreDeclaredPublic => {
                    self.assertion_results.push_expected("are declared public");
                    match_against.structs_that(Struct::is_public)
                }
                ConditionToken::AreDeclaredPrivate => {
                    self.assertion_results.push_expected("are declared private");
                    match_against.structs_that(|struct_| !struct_.is_public())
                }
                ConditionToken::HaveSimpleName(name) => {
                    self.assertion_results
                        .push_expected(format!("have simple name '{name}'"));
                    match_against.structs_that(|struct_| struct_.ident == name)
                }
                ConditionToken::HaveNameMatching(pattern) => {
                    self.assertion_results
                        .push_expected(format!("have name matching '{pattern}'"));
                    match_against
                        .structs_that(|struct_| WildMatch::new(&pattern).matches(&struct_.ident))
                }
                ConditionToken::ResidesInAModule(name) => {
                    self.assertion_results
                        .push_expected(format!("resides in a modules that match '{name}'"));
                    match_against.structs_that(|struct_| struct_.path_match(&name))
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
                ConditionToken::Derives(trait_) => {
                    let expected = format!("derive {trait_}");
                    self.assertion_results.push_expected(&expected);
                    match_against.structs_that(|struct_| struct_.derives(&trait_))
                }
                ConditionToken::Implement(trait_) => {
                    let expected = format!("implement {trait_}");
                    self.assertion_results.push_expected(&expected);
                    let imps = impl_matches(&self.filters)
                        .impl_that(|imp| matches!(&imp.trait_impl, Some(t) if t.contains(&trait_)));
                    let types = imps.types();
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
                    SimpleAssertions::HaveSimpleName(name) => self.assert_simple_name(&name),
                    SimpleAssertions::Implement(trait_) => self.assert_implement(&trait_),
                    SimpleAssertions::Derive(trait_) => self.assert_derives(&trait_),
                    SimpleAssertions::ImplementOrDerive(trait_) => {
                        self.assert_implement_or_derive(&trait_)
                    }
                    SimpleAssertions::OnlyHavePrivateFields => self.assert_private_fields(),
                    SimpleAssertions::OnlyHavePublicFields => self.assert_public_fields(),
                },
                AssertionToken::Conjunction(a) => match a {
                    AssertionConjunction::AndShould => {
                        self.assertion_results.push_expected(" and ");
                        conjunction = Conjunction::And;
                        true
                    }
                    AssertionConjunction::OrShould => {
                        self.assertion_results.push_expected(" or ");
                        conjunction = Conjunction::Or;
                        true
                    }
                },
            };

            match conjunction {
                Conjunction::Or => {
                    success = success || assertion_outcome;
                    // Make sure we use logical && until the next conjunction
                    conjunction = Conjunction::And;
                }
                Conjunction::And => success = success && assertion_outcome,
            };
        }
    }

    fn assertion_results(&self) -> &AssertionResult {
        &self.assertion_results
    }
}

impl ArchRule<ConditionToken, AssertionToken, StructMatches> {
    fn assert_public(&mut self) -> bool {
        self.assertion_results.push_expected("be public");
        let non_public_struct = self
            .subject
            .0
            .iter()
            .filter(|struct_| !struct_.is_public())
            .collect::<Vec<_>>();

        for struct_ in &non_public_struct {
            self.assertion_results
                .push_actual(StructRuleViolation::be_public(
                    struct_.span,
                    &struct_.real_path,
                    struct_.ident.clone(),
                    struct_.visibility,
                ));
        }
        non_public_struct.is_empty()
    }

    fn assert_private(&mut self) -> bool {
        self.assertion_results.push_expected("be private");
        let public_structs = self
            .subject
            .0
            .iter()
            .filter(|struct_| struct_.is_public())
            .collect::<Vec<_>>();

        for struct_ in &public_structs {
            self.assertion_results
                .push_actual(StructRuleViolation::be_private(
                    struct_.span,
                    &struct_.real_path,
                    struct_.ident.clone(),
                    struct_.visibility,
                ));
        }

        public_structs.is_empty()
    }

    fn assert_simple_name(&mut self, name: &str) -> bool {
        self.assertion_results
            .push_expected(format!("have simple name '{name}'"));
        let struct_with_non_matching_name = self
            .subject
            .0
            .iter()
            .filter(|struct_| struct_.ident != name)
            .collect::<Vec<_>>();

        for struct_ in &struct_with_non_matching_name {
            self.assertion_results
                .push_actual(StructRuleViolation::have_name_matching(
                    struct_.span,
                    &struct_.real_path,
                    struct_.ident.clone(),
                    name.to_owned(),
                ));
        }

        struct_with_non_matching_name.is_empty()
    }

    fn assert_derives(&mut self, trait_: &String) -> bool {
        self.assertion_results
            .push_expected(format!("derive '{trait_}'"));

        let struct_without_expected_derive = self
            .subject
            .0
            .iter()
            .filter(|struct_| !struct_.derives.contains(trait_))
            .collect::<Vec<_>>();

        for struct_ in &struct_without_expected_derive {
            self.assertion_results
                .push_actual(StructRuleViolation::derive(
                    struct_.span,
                    &struct_.real_path,
                    struct_.ident.clone(),
                    trait_.to_string(),
                ));
        }

        struct_without_expected_derive.is_empty()
    }

    fn assert_implement(&mut self, trait_: &String) -> bool {
        self.assertion_results
            .push_expected(format!("implement '{trait_}'"));

        let struct_without_expected_impl = self
            .subject
            .0
            .iter()
            .filter(|struct_| {
                let imp_for_type = impl_matches(&self.filters)
                    .impl_that(|imp| imp.self_ty.name() == struct_.ident.as_str());

                let imp_for_type = imp_for_type
                    .impl_that(|imp| matches!(&imp.trait_impl, Some(t) if t.contains(trait_)));

                imp_for_type.is_empty()
            })
            .collect::<Vec<_>>();

        for struct_ in &struct_without_expected_impl {
            self.assertion_results
                .push_actual(StructRuleViolation::implement(
                    struct_.span,
                    &struct_.real_path,
                    struct_.ident.clone(),
                    trait_.to_string(),
                ));
        }

        struct_without_expected_impl.is_empty()
    }

    fn assert_implement_or_derive(&mut self, trait_: &String) -> bool {
        self.assertion_results
            .push_expected(format!("derive '{trait_}'"));

        let derive_set = self
            .subject
            .0
            .iter()
            .filter(|struct_| !struct_.derives.contains(trait_))
            .collect::<HashSet<_>>();

        let impl_set = self
            .subject
            .0
            .iter()
            .filter(|struct_| {
                let imp_for_type = impl_matches(&self.filters)
                    .impl_that(|imp| imp.self_ty.name() == struct_.ident.as_str());

                let imp_for_type = imp_for_type
                    .impl_that(|imp| matches!(&imp.trait_impl, Some(t) if t.contains(trait_)));

                imp_for_type.is_empty()
            })
            .collect::<HashSet<_>>();

        let intersection: Vec<&&&Struct> = impl_set.intersection(&derive_set).collect();
        for struct_ in &intersection {
            self.assertion_results
                .push_actual(StructRuleViolation::implement_or_derive(
                    struct_.span,
                    &struct_.real_path,
                    struct_.ident.clone(),
                    trait_.to_string(),
                ));
        }

        intersection.is_empty()
    }

    fn assert_private_fields(&mut self) -> bool {
        self.assertion_results
            .push_expected("only have private fields");
        let struct_with_only_public_fields = self
            .subject
            .0
            .iter()
            .filter(|struct_| !struct_.fields.is_empty())
            .filter(|struct_| !struct_.has_non_public_field())
            .collect::<Vec<_>>();

        for struct_ in &struct_with_only_public_fields {
            StructRuleViolation::only_have_private_fields(
                &struct_.real_path,
                struct_.ident.clone(),
                &struct_.fields,
            )
            .into_iter()
            .for_each(|report| self.assertion_results.push_actual(report));
        }
        struct_with_only_public_fields.is_empty()
    }

    fn assert_public_fields(&mut self) -> bool {
        self.assertion_results
            .push_expected("only have public fields");
        let struct_with_non_public_fields = self
            .subject
            .0
            .iter()
            .filter(|struct_| !struct_.fields.is_empty())
            .filter(|struct_| struct_.has_non_public_field())
            .collect::<Vec<_>>();

        for struct_ in &struct_with_non_public_fields {
            StructRuleViolation::only_have_public_fields(
                &struct_.real_path,
                struct_.ident.clone(),
                &struct_.fields,
            )
            .into_iter()
            .for_each(|report| self.assertion_results.push_actual(report));
        }

        struct_with_non_public_fields.is_empty()
    }
}

#[cfg(test)]
mod condition_test {
    use crate::rule::structs::Structs;
    use crate::rule::{ArchRuleBuilder, CheckRule};
    use crate::Filters;

    #[test]
    #[should_panic]
    fn should_check_derives_panic() {
        Structs::that(Filters::default())
            .reside_in_a_module("*::modules")
            .should()
            .derive("Eq")
            .check();
    }

    #[test]
    fn should_check_derives_ok() {
        Structs::that(Filters::default())
            .reside_in_a_module("assertion_result")
            .should()
            .derive("Debug")
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_private_fields() {
        Structs::that(Filters::default())
            .are_declared_public()
            .should()
            .only_have_private_fields()
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_private_fields_for_unamed_fields() {
        Structs::that(Filters::default())
            .have_simple_name("ModuleMatches")
            .should()
            .only_have_private_fields()
            .check();
    }

    #[test]
    fn should_check_public_fields_ok() {
        Structs::that(Filters::default())
            .reside_in_a_module("assertion_result")
            .should()
            .only_have_public_fields()
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_public_fields_err() {
        Structs::that(Filters::default())
            .have_simple_name("ItemPath")
            .or()
            .have_simple_name("CodeSpan")
            .should()
            .only_have_public_fields()
            .check();
    }

    #[test]
    #[should_panic]
    fn should_filter_implementors() {
        Structs::that(Filters::default())
            .implement("Display")
            .should()
            .be_private()
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_implementation() {
        Structs::that(Filters::default())
            .have_simple_name("AssertionResult")
            .should()
            .implement("Debug")
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_derive() {
        Structs::that(Filters::default())
            .have_simple_name("AssertionResult")
            .should()
            .derive("PartialEq")
            .check();
    }

    #[test]
    #[should_panic]
    fn struct_suffixed_with_matches_should_implement_subject_panic() {
        Structs::that(Filters::default())
            .have_simple_name("EnumMatches")
            .or()
            .have_simple_name("Modules")
            .should()
            .implement("Subject")
            .check();
    }

    #[test]
    fn all_structs_should_derive_debug() {
        Structs::all_should(Filters::default())
            .derive("Debug")
            .check();
    }

    #[test]
    fn all_structs_should_derive_or_implement_debug() {
        Structs::all_should(Filters::default())
            .implement_or_derive("Debug")
            .check();
    }

    #[test]
    fn structs_by_name_matching_should_implement_subject() {
        Structs::that(Filters::default())
            .have_name_matching("*Matches")
            .should()
            .implement("Subject")
            .check();
    }

    #[test]
    fn structs_by_simple_name_should_implement_subject() {
        Structs::that(Filters::default())
            .have_simple_name("EnumMatches")
            .or()
            .have_simple_name("ModuleMatches")
            .or()
            .have_simple_name("StructMatches")
            .should()
            .implement("Subject")
            .check();
    }
}
