use crate::ast::{ItemPath, ModuleUse};
use crate::rule::{
    ArchRuleBuilder, Assertion, Condition, ConditionBuilder, ConditionConjunctionBuilder,
    DependencyPredicateConjunctionBuilder, PredicateBuilder, PredicateConjunctionBuilder, Subject,
};
use crate::ModuleTree;
use std::collections::HashMap;
use std::path::PathBuf;

mod check;
mod condition;
mod report;

/// A unit struct giving access to module assertions:
/// **Example:**
/// ```rust
/// use archunit_rs::Filters;
/// use archunit_rs::rule::{ArchRuleBuilder, CheckRule};
/// use archunit_rs::rule::modules::Modules;
///
/// Modules::that(Filters::default())
///     .have_simple_name("archunit_rs")
///     .should()
///     .be_public()
///     .check();
/// ```
#[derive(Debug)]
pub struct Modules;

#[derive(Default, Debug)]
pub struct ModuleMatches(pub HashMap<&'static ItemPath, &'static ModuleTree>);

#[derive(Default, Debug)]
pub struct ModuleDependencies(
    pub HashMap<&'static ItemPath, (&'static PathBuf, &'static Vec<ModuleUse>)>,
);

impl ModuleMatches {
    pub fn extend(&mut self, other: ModuleMatches) {
        self.0.extend(other.0);
    }
}

pub type ModuleConditionBuilder = ConditionBuilder<ConditionToken, AssertionToken, ModuleMatches>;
pub type ModuleConditionConjunctionBuilder =
    ConditionConjunctionBuilder<ConditionToken, AssertionToken, ModuleMatches>;
pub type ModulePredicateBuilder = PredicateBuilder<ConditionToken, AssertionToken, ModuleMatches>;
pub type ModuleDependencyPredicateConjunctionBuilder =
    DependencyPredicateConjunctionBuilder<ConditionToken, AssertionToken, ModuleMatches>;
pub type ModulePredicateConjunctionBuilder =
    PredicateConjunctionBuilder<ConditionToken, AssertionToken, ModuleMatches>;

impl Condition for ConditionToken {}

impl Assertion for AssertionToken {}

impl Subject for ModuleMatches {}

#[derive(Debug, PartialEq, Eq)]
pub enum ConditionToken {
    AreDeclaredPublic,
    ResidesInAModule(String),
    AreDeclaredPrivate,
    HaveSimpleName(String),
    HaveSimpleEndingWith(String),
    HaveSimpleStartingWith(String),
    And,
    Or,
    Should,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssertionToken {
    SimpleAssertion(SimpleAssertions),
    Conjunction(AssertionConjunction),
    DependencyAssertion(DependencyAssertion),
    DependencyAssertionConjunction(DependencyAssertionConjunction),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DependencyAssertionConjunction {
    OnlyHaveDependencyModule,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DependencyAssertion {
    That,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SimpleAssertions {
    BePublic,
    BePrivate,
    HaveSimpleName(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssertionConjunction {
    AndShould,
    OrShould,
}

impl ArchRuleBuilder<ConditionToken, AssertionToken, ModuleMatches> for Modules {}

impl ModuleConditionBuilder {
    pub fn reside_in_a_module(mut self, module: &str) -> ModuleConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::ResidesInAModule(module.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }

    pub fn are_declared_public(mut self) -> ModuleConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::AreDeclaredPublic);
        ConditionConjunctionBuilder(self.0)
    }

    pub fn are_declared_private(mut self) -> ModuleConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::AreDeclaredPrivate);
        ConditionConjunctionBuilder(self.0)
    }

    pub fn have_simple_name(mut self, name: &str) -> ModuleConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::HaveSimpleName(name.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }
}

impl ModuleConditionConjunctionBuilder {
    pub fn and(mut self) -> ModuleConditionBuilder {
        self.0.conditions.push_front(ConditionToken::And);
        ConditionBuilder(self.0)
    }

    pub fn or(mut self) -> ModuleConditionBuilder {
        self.0.conditions.push_front(ConditionToken::Or);
        ConditionBuilder(self.0)
    }

    pub fn should(mut self) -> ModulePredicateBuilder {
        self.0.conditions.push_front(ConditionToken::Should);
        PredicateBuilder(self.0)
    }
}

impl ModulePredicateBuilder {
    pub fn have_simple_name(mut self, name: &str) -> ModulePredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::HaveSimpleName(name.to_owned()),
            ));
        PredicateConjunctionBuilder(self.0)
    }

    pub fn be_public(mut self) -> ModulePredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::BePublic));
        PredicateConjunctionBuilder(self.0)
    }

    pub fn be_private(mut self) -> ModulePredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::BePrivate));
        PredicateConjunctionBuilder(self.0)
    }

    pub fn only_have_dependency_module(mut self) -> ModuleDependencyPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::DependencyAssertionConjunction(
                DependencyAssertionConjunction::OnlyHaveDependencyModule,
            ));
        DependencyPredicateConjunctionBuilder(self.0)
    }
}

impl ModulePredicateConjunctionBuilder {
    pub fn and_should(mut self) -> ModulePredicateBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::Conjunction(AssertionConjunction::AndShould));
        PredicateBuilder(self.0)
    }

    pub fn or_should(mut self) -> ModulePredicateBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::Conjunction(AssertionConjunction::OrShould));
        PredicateBuilder(self.0)
    }
}

impl ModuleDependencyPredicateConjunctionBuilder {
    pub fn that(mut self) -> PredicateBuilder<ConditionToken, AssertionToken, ModuleMatches> {
        self.0
            .assertions
            .push_front(AssertionToken::DependencyAssertion(
                DependencyAssertion::That,
            ));

        PredicateBuilder(self.0)
    }
}

#[cfg(test)]
mod module_test {
    use crate::rule::modules::{
        AssertionConjunction, AssertionToken, ConditionToken, DependencyAssertion,
        DependencyAssertionConjunction, Modules, SimpleAssertions,
    };
    use crate::rule::ArchRuleBuilder;
    use crate::Filters;
    use speculoos::prelude::*;

    #[test]
    fn should_build_arch_rule_for_module() {
        let rule = Modules::that(Filters::default())
            .reside_in_a_module("foo::bar")
            .and()
            .are_declared_private()
            .or()
            .are_declared_public()
            .should()
            .only_have_dependency_module()
            .that()
            .have_simple_name("toto")
            .and_should()
            .be_public()
            .or_should()
            .be_private();

        assert_that!(rule.0.conditions.iter()).equals_iterator(
            &[
                ConditionToken::Should,
                ConditionToken::AreDeclaredPublic,
                ConditionToken::Or,
                ConditionToken::AreDeclaredPrivate,
                ConditionToken::And,
                ConditionToken::ResidesInAModule("foo::bar".to_owned()),
            ]
            .iter(),
        );

        assert_that!(rule.0.assertions.iter()).equals_iterator(
            &[
                AssertionToken::SimpleAssertion(SimpleAssertions::BePrivate),
                AssertionToken::Conjunction(AssertionConjunction::OrShould),
                AssertionToken::SimpleAssertion(SimpleAssertions::BePublic),
                AssertionToken::Conjunction(AssertionConjunction::AndShould),
                AssertionToken::SimpleAssertion(SimpleAssertions::HaveSimpleName(
                    "toto".to_owned(),
                )),
                AssertionToken::DependencyAssertion(DependencyAssertion::That),
                AssertionToken::DependencyAssertionConjunction(
                    DependencyAssertionConjunction::OnlyHaveDependencyModule,
                ),
            ]
            .iter(),
        )
    }
}
