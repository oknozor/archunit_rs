mod check;
mod condition;
mod reports;

use crate::ast::enums::Enum;
use crate::ast::module_tree;
use crate::rule::{
    ArchRuleBuilder, Assertion, Condition, ConditionBuilder, ConditionConjunctionBuilder,
    PredicateBuilder, PredicateConjunctionBuilder, Subject,
};
use crate::ExludeModules;
use std::collections::HashSet;

/// A unit enum giving access to enum assertions.
///
/// **Example:**
/// ```rust
/// use archunit_rs::ExludeModules;
/// use archunit_rs::rule::ArchRuleBuilder;
/// use archunit_rs::rule::enums::Enums;
///
/// Enums::that(ExludeModules::default())
///     .have_simple_name("ConditionToken")
///     .should()
///     .be_public();
/// ```
#[derive(Debug)]
pub struct Enums;

#[derive(Debug, Default)]
pub struct EnumMatches(pub(crate) HashSet<&'static Enum>);

impl EnumMatches {
    pub fn enums_that<P>(&self, mut predicate: P) -> EnumMatches
    where
        P: FnMut(&Enum) -> bool,
    {
        let mut set = HashSet::new();
        self.0
            .iter()
            .copied()
            .filter(|enum_| predicate(enum_))
            .for_each(|enum_| {
                set.insert(enum_);
            });

        EnumMatches(set)
    }

    pub fn extends(&mut self, other: EnumMatches) {
        self.0.extend(other.0)
    }
}

impl ArchRuleBuilder<ConditionToken, AssertionToken, EnumMatches> for Enums {}

/// Type alias for `[ConditionBuilder]` enum implementation.
pub type EnumConditionBuilder = ConditionBuilder<ConditionToken, AssertionToken, EnumMatches>;

/// Type alias for`[ConditionConjunctionBuilder]` enum implementation.
pub type EnumConditionConjunctionBuilder =
    ConditionConjunctionBuilder<ConditionToken, AssertionToken, EnumMatches>;

/// Type alias for `[PredicateBuilder]` enum implementation.
pub type EnumPredicateBuilder = PredicateBuilder<ConditionToken, AssertionToken, EnumMatches>;

/// Type alias for `[PredicateConjunctionBuilder]` enum implementation.
pub type EnumPredicateConjunctionBuilder =
    PredicateConjunctionBuilder<ConditionToken, AssertionToken, EnumMatches>;

impl Condition for ConditionToken {}

impl Assertion for AssertionToken {}

impl Subject for EnumMatches {
    fn init(filters: &ExludeModules<'static>) -> Self {
        module_tree().flatten_enums(filters)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConditionToken {
    AreDeclaredPublic,
    ResidesInAModule(String),
    AreDeclaredPrivate,
    HaveSimpleName(String),
    Derives(String),
    Implement(String),
    And,
    Or,
    Should,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssertionToken {
    SimpleAssertion(SimpleAssertions),
    Conjunction(AssertionConjunction),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SimpleAssertions {
    BePublic,
    BePrivate,
    HaveSimpleName(String),
    Implement(String),
    ImplementOrDerive(String),
    Derive(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssertionConjunction {
    AndShould,
    OrShould,
}

impl EnumConditionBuilder {
    /// filter enum that resides in the given module
    pub fn reside_in_a_module(mut self, module: &str) -> EnumConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::ResidesInAModule(module.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }

    /// filter enum that are declared public
    pub fn are_declared_public(mut self) -> EnumConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::AreDeclaredPublic);
        ConditionConjunctionBuilder(self.0)
    }

    /// filter enum with restricted visibility
    pub fn are_declared_private(mut self) -> EnumConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::AreDeclaredPrivate);
        ConditionConjunctionBuilder(self.0)
    }

    /// filter enum with the given name
    pub fn have_simple_name(mut self, name: &str) -> EnumConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::HaveSimpleName(name.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }

    /// filter enum that derives the given trait
    pub fn derives(mut self, trait_name: &str) -> EnumConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::Derives(trait_name.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }

    /// filter enum that implement the given trait
    pub fn implement(mut self, trait_name: &str) -> EnumConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::Implement(trait_name.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }
}

impl EnumConditionConjunctionBuilder {
    /// `And` conjunction for enum conditions.
    pub fn and(mut self) -> EnumConditionBuilder {
        self.0.conditions.push_front(ConditionToken::And);
        ConditionBuilder(self.0)
    }

    /// `Or` conjunction for enum conditions.
    pub fn or(mut self) -> EnumConditionBuilder {
        self.0.conditions.push_front(ConditionToken::Or);
        ConditionBuilder(self.0)
    }

    /// Apply the current conditions.
    pub fn should(mut self) -> EnumPredicateBuilder {
        self.0.conditions.push_front(ConditionToken::Should);
        PredicateBuilder(self.0)
    }
}

impl EnumPredicateBuilder {
    /// Predicate matching structs with the given name.
    pub fn have_simple_name(mut self, name: &str) -> EnumPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::HaveSimpleName(name.to_owned()),
            ));
        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching public structs.
    pub fn be_public(mut self) -> EnumPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::BePublic));
        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching private structs.
    pub fn be_private(mut self) -> EnumPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::BePrivate));
        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching enum implementing the given trait.
    pub fn implement(mut self, trait_name: &str) -> EnumPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::Implement(trait_name.to_owned()),
            ));

        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching enum which derives the given trait.
    pub fn derive(mut self, trait_name: &str) -> EnumPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::Derive(
                trait_name.to_owned(),
            )));

        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching enum that implement or derive the given trait.
    pub fn implement_or_derive(mut self, trait_name: &str) -> EnumPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::ImplementOrDerive(trait_name.to_owned()),
            ));

        PredicateConjunctionBuilder(self.0)
    }
}

impl EnumPredicateConjunctionBuilder {
    /// Combine two predicate with`And` conjunction.
    pub fn and_should(mut self) -> EnumPredicateBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::Conjunction(AssertionConjunction::AndShould));
        PredicateBuilder(self.0)
    }

    /// Combine two predicate with the `Or` conjunction.
    pub fn or_should(mut self) -> EnumPredicateBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::Conjunction(AssertionConjunction::OrShould));
        PredicateBuilder(self.0)
    }
}

#[cfg(test)]
mod test {
    use crate::rule::enums::{
        AssertionConjunction, AssertionToken, ConditionToken, Enums, SimpleAssertions,
    };
    use crate::rule::ArchRuleBuilder;
    use crate::ExludeModules;
    use speculoos::prelude::*;

    #[test]
    fn should_build_arch_rule_for_struct() {
        let rule = Enums::that(ExludeModules::default())
            .derives("Debug")
            .and()
            .implement("Display")
            .or()
            .reside_in_a_module("::check")
            .and()
            .are_declared_private()
            .and()
            .have_simple_name("Name")
            .should()
            .implement("Name")
            .or_should()
            .be_public()
            .and_should()
            .have_simple_name("Name")
            .or_should()
            .be_private();

        assert_that!(rule.0.conditions.iter()).equals_iterator(
            &[
                ConditionToken::Should,
                ConditionToken::HaveSimpleName("Name".to_owned()),
                ConditionToken::And,
                ConditionToken::AreDeclaredPrivate,
                ConditionToken::And,
                ConditionToken::ResidesInAModule("::check".to_owned()),
                ConditionToken::Or,
                ConditionToken::Implement("Display".to_owned()),
                ConditionToken::And,
                ConditionToken::Derives("Debug".to_owned()),
            ]
            .iter(),
        );

        assert_that!(rule.0.assertions.iter()).equals_iterator(
            &[
                AssertionToken::SimpleAssertion(SimpleAssertions::BePrivate),
                AssertionToken::Conjunction(AssertionConjunction::OrShould),
                AssertionToken::SimpleAssertion(SimpleAssertions::HaveSimpleName(
                    "Name".to_owned(),
                )),
                AssertionToken::Conjunction(AssertionConjunction::AndShould),
                AssertionToken::SimpleAssertion(SimpleAssertions::BePublic),
                AssertionToken::Conjunction(AssertionConjunction::OrShould),
                AssertionToken::SimpleAssertion(SimpleAssertions::Implement("Name".to_owned())),
            ]
            .iter(),
        )
    }
}
