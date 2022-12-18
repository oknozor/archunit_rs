use crate::assertion_result::AssertionResult;
use crate::ast::enums::Enum;
use crate::rule::assertable::Assertable;
use crate::rule::enums::reports::EnumRuleViolation;
use crate::rule::enums::{
    AssertionConjunction, AssertionToken, ConditionToken, EnumMatches,
    EnumPredicateConjunctionBuilder, SimpleAssertions,
};
use crate::rule::impl_block::impl_matches;
use crate::rule::{ArchRule, CheckRule};
use std::collections::HashSet;

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
        let enums = self.init_subject();

        enum Conjunction {
            Or,
            And,
        }

        let mut conjunction = Conjunction::Or;
        self.assertion_results.push_expected("Structs that ");
        while let Some(condition) = self.conditions.pop_back() {
            let match_against = match conjunction {
                Conjunction::Or => &enums,
                Conjunction::And => &matches,
            };

            let matches_for_condition = match condition {
                ConditionToken::AreDeclaredPublic => {
                    self.assertion_results.push_expected("are declared public");
                    match_against.enums_that(Enum::is_public)
                }
                ConditionToken::AreDeclaredPrivate => {
                    self.assertion_results.push_expected("are declared private");
                    match_against.enums_that(|enum_| !enum_.is_public())
                }
                ConditionToken::HaveSimpleName(name) => {
                    self.assertion_results
                        .push_expected(format!("have simple name '{name}'"));
                    match_against.enums_that(|enum_| enum_.ident == name)
                }
                ConditionToken::ResidesInAModule(name) => {
                    self.assertion_results
                        .push_expected(format!("resides in a modules that match '{name}'"));
                    match_against.enums_that(|enum_| enum_.path_match(&name))
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
                    match_against.enums_that(|enum_| enum_.derives(&trait_))
                }
                ConditionToken::Implement(trait_) => {
                    let expected = format!("implement {trait_}");
                    self.assertion_results.push_expected(&expected);
                    let imps = impl_matches(&self.filters)
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
                    SimpleAssertions::Implement(trait_) => self.assert_implement(&trait_),
                    SimpleAssertions::ImplementOrDerive(trait_) => {
                        self.assert_implement_or_derive(&trait_)
                    }
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

        success
    }

    fn assertion_results(&self) -> &AssertionResult {
        &self.assertion_results
    }

    fn has_conditions(&self) -> bool {
        !self.conditions.is_empty()
    }
}

impl ArchRule<ConditionToken, AssertionToken, EnumMatches> {
    fn assert_public(&mut self) -> bool {
        self.assertion_results.push_expected("be public");
        let non_public_struct = self
            .subject
            .0
            .iter()
            .filter(|enum_| !enum_.is_public())
            .collect::<Vec<_>>();

        for enum_ in &non_public_struct {
            self.assertion_results
                .push_actual(EnumRuleViolation::be_public(
                    enum_.span,
                    &enum_.location,
                    enum_.ident.clone(),
                    enum_.visibility,
                ));
        }

        non_public_struct.is_empty()
    }

    fn assert_private(&mut self) -> bool {
        self.assertion_results.push_expected("be private");
        let public_enum = self
            .subject
            .0
            .iter()
            .filter(|enum_| enum_.is_public())
            .collect::<Vec<_>>();

        for enum_ in &public_enum {
            self.assertion_results
                .push_actual(EnumRuleViolation::be_private(
                    enum_.span,
                    &enum_.location,
                    enum_.ident.clone(),
                    enum_.visibility,
                ))
        }

        public_enum.is_empty()
    }

    fn assert_simple_name(&mut self, name: &str) -> bool {
        self.assertion_results
            .push_expected(format!("have simple name '{name}'"));
        let enum_with_non_matching_name = self
            .subject
            .0
            .iter()
            .filter(|enum_| enum_.ident != name)
            .collect::<Vec<_>>();

        for enum_ in &enum_with_non_matching_name {
            self.assertion_results
                .push_actual(EnumRuleViolation::have_simple(
                    enum_.span,
                    name.to_owned(),
                    &enum_.location,
                    enum_.ident.clone(),
                ))
        }

        enum_with_non_matching_name.is_empty()
    }

    fn assert_derives(&mut self, trait_: &String) -> bool {
        self.assertion_results
            .push_expected(format!("derive '{trait_}'"));

        let enum_without_expected_derive = self
            .subject
            .0
            .iter()
            .filter(|enum_| !enum_.derives.contains(trait_))
            .collect::<Vec<_>>();

        for enum_ in &enum_without_expected_derive {
            self.assertion_results
                .push_actual(EnumRuleViolation::derive(
                    enum_.span,
                    &enum_.location,
                    enum_.ident.clone(),
                    trait_.clone(),
                ))
        }

        enum_without_expected_derive.is_empty()
    }

    fn assert_implement(&mut self, trait_: &String) -> bool {
        self.assertion_results
            .push_expected(format!("implement '{trait_}'"));

        let enum_without_expected_impl = self
            .subject
            .0
            .iter()
            .filter(|enum_| {
                let imp_for_type = impl_matches(&self.filters)
                    .impl_that(|imp| imp.self_ty.name() == enum_.ident.as_str());
                let imp_for_type = imp_for_type
                    .impl_that(|imp| matches!(&imp.trait_impl, Some(t) if t.contains(trait_)));
                imp_for_type.is_empty()
            })
            .collect::<Vec<_>>();

        for enum_ in &enum_without_expected_impl {
            self.assertion_results
                .push_actual(EnumRuleViolation::implement(
                    enum_.span,
                    &enum_.location,
                    enum_.ident.clone(),
                    trait_.clone(),
                ))
        }

        enum_without_expected_impl.is_empty()
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

        let intersection: Vec<&&&Enum> = impl_set.intersection(&derive_set).collect();
        for enum_ in &intersection {
            self.assertion_results
                .push_actual(EnumRuleViolation::implement_or_derive(
                    enum_.span,
                    &enum_.location,
                    enum_.ident.clone(),
                    trait_.clone(),
                ))
        }

        intersection.is_empty()
    }
}

#[cfg(test)]
mod condition_test {
    use crate::rule::enums::Enums;
    use crate::rule::{ArchRuleBuilder, CheckRule};
    use crate::ExludeModules;

    #[test]
    #[should_panic(
        expected = r#"Expected Structs that resides in a modules that match '*::report' to derive 'Deserialize' but found 1 violations

  × Enum 'ModuleRuleViolation' should derive 'Deserialize'
   ╭─[src/rule/modules/report.rs:1:1]
 1 │ #[derive(Error, Debug, Diagnostic)]
 2 │ pub(crate) enum ModuleRuleViolation {
   ·                 ─────────┬─────────
   ·                          ╰── missing derive
 3 │     #[error("Module '{module_name}' should be private")]
   ╰────
  help: Try adding `#[derive(Deserialize)]` to `ModuleRuleViolation`

"#
    )]
    fn should_panic_when_enum_does_not_derive() {
        Enums::that(ExludeModules::default())
            .reside_in_a_module("*::report")
            .should()
            .derive("Deserialize")
            .check();
    }

    #[test]
    fn should_not_panic_when_enum_does_derive() {
        Enums::that(ExludeModules::default())
            .have_simple_name("Visibility")
            .should()
            .derive("Debug")
            .check();
    }

    #[test]
    fn should_not_panic_when_implementors_have_simple_name() {
        Enums::that(ExludeModules::default())
            .implement("Condition")
            .should()
            .have_simple_name("ConditionToken")
            .check();
    }

    #[test]
    #[should_panic]
    fn should_panic_when_implementors_does_not_have_simple_name() {
        Enums::that(ExludeModules::default())
            .implement("Condition")
            .should()
            .have_simple_name("AssertionToken")
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_implementation() {
        Enums::that(ExludeModules::default())
            .have_simple_name("AssertionToken")
            .should()
            .implement("Debug")
            .check();
    }

    #[test]
    #[should_panic]
    fn should_check_with_or_condition_operator() {
        Enums::that(ExludeModules::default())
            .have_simple_name("ConditionToken")
            .or()
            .have_simple_name("AssertionResult")
            .should()
            .derive("Clone")
            .check();
    }

    #[test]
    fn should_check_with_or_assertion_operator() {
        Enums::that(ExludeModules::default())
            .have_simple_name("AssertionToken")
            .should()
            .derive("Debug")
            .or_should()
            .derive("PartialOrd")
            .check();
    }

    #[test]
    fn should_derive_or_implement_debug_ok() {
        // Note: we are currently limited to struct and enum living in modules
        // anything living inside a function is ignored
        Enums::all_should(ExludeModules::default())
            .implement_or_derive("Debug")
            .check();
    }

    #[test]
    #[should_panic]
    fn should_panic_derive_or_implement_ord() {
        // Note: we are currently limited to struct and enum living in modules
        // anything living inside a function is ignored
        Enums::all_should(ExludeModules::default())
            .implement_or_derive("Ord")
            .check();
    }
}
