use crate::ast::module_tree;
use crate::ast::structs::Struct;
use crate::rule::{
    ArchRuleBuilder, Assertion, Condition, ConditionBuilder, ConditionConjunctionBuilder,
    PredicateBuilder, PredicateConjunctionBuilder, Subject,
};
use crate::Filters;
use std::collections::HashSet;

pub mod check;
pub mod condition;
pub mod reports;

/// A unit struct giving access to struct assertions.
///
/// **Example:**
/// ```rust
/// use archunit_rs::Filters;
/// use archunit_rs::rule::ArchRuleBuilder;
/// use archunit_rs::rule::structs::Structs;
///
/// Structs::that(Filters::default())
/// .have_simple_name("PredicateBuilder")
/// .should()
/// .only_have_private_fields()
/// .and_should()
/// .be_public();
/// ```
#[derive(Debug)]
pub struct Structs;

#[derive(Debug, Default)]
pub struct StructMatches(pub(crate) HashSet<&'static Struct>);

impl StructMatches {
    pub fn structs_that<P>(&self, mut predicate: P) -> StructMatches
    where
        P: FnMut(&Struct) -> bool,
    {
        let mut set = HashSet::new();
        self.0
            .iter()
            .copied()
            .filter(|struct_| predicate(struct_))
            .for_each(|struct_| {
                set.insert(struct_);
            });

        StructMatches(set)
    }

    pub fn extends(&mut self, other: StructMatches) {
        self.0.extend(other.0)
    }
}

impl ArchRuleBuilder<ConditionToken, AssertionToken, StructMatches> for Structs {}

/// Type alias for `[ConditionBuilder]` struct implementation.
pub type StructConditionBuilder = ConditionBuilder<ConditionToken, AssertionToken, StructMatches>;

/// Type alias for`[ConditionConjunctionBuilder]` struct implementation.
pub type StructConditionConjunctionBuilder =
    ConditionConjunctionBuilder<ConditionToken, AssertionToken, StructMatches>;

/// Type alias for `[PredicateBuilder]` struct implementation.
pub type StructPredicateBuilder = PredicateBuilder<ConditionToken, AssertionToken, StructMatches>;

/// Type alias for `[PredicateConjunctionBuilder]` struct implementation.
pub type StructPredicateConjunctionBuilder =
    PredicateConjunctionBuilder<ConditionToken, AssertionToken, StructMatches>;

impl Condition for ConditionToken {}

impl Assertion for AssertionToken {}

impl Subject for StructMatches {
    fn init(filters: &Filters<'static>) -> Self {
        module_tree().flatten_structs(filters)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConditionToken {
    AreDeclaredPublic,
    ResidesInAModule(String),
    AreDeclaredPrivate,
    HaveSimpleName(String),
    HaveNameMatching(String),
    Derives(String),
    Implement(String),
    And,
    Or,
    Should,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Derive(String),
    ImplementOrDerive(String),
    OnlyHavePrivateFields,
    OnlyHavePublicFields,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssertionConjunction {
    AndShould,
    OrShould,
}

impl StructConditionBuilder {
    /// filter struct that resides in the given module
    pub fn reside_in_a_module(mut self, module: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::ResidesInAModule(module.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }

    /// filter struct that are declared public
    pub fn are_declared_public(mut self) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::AreDeclaredPublic);
        ConditionConjunctionBuilder(self.0)
    }

    /// filter struct with restricted visibility
    pub fn are_declared_private(mut self) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::AreDeclaredPrivate);
        ConditionConjunctionBuilder(self.0)
    }

    /// filter struct with the given name
    pub fn have_simple_name(mut self, name: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::HaveSimpleName(name.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }

    /// filter struct with the given name
    pub fn have_name_matching(mut self, pattern: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::HaveNameMatching(pattern.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }

    /// filter struct that derives the given trait
    pub fn derives(mut self, trait_name: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::Derives(trait_name.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }

    /// filter struct that implement the given trait
    pub fn implement(mut self, trait_name: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::Implement(trait_name.to_owned()));
        ConditionConjunctionBuilder(self.0)
    }
}

impl StructConditionConjunctionBuilder {
    /// `And` conjunction for struct conditions.
    pub fn and(mut self) -> StructConditionBuilder {
        self.0.conditions.push_front(ConditionToken::And);
        ConditionBuilder(self.0)
    }

    /// `Or` conjunction for struct conditions.
    pub fn or(mut self) -> StructConditionBuilder {
        self.0.conditions.push_front(ConditionToken::Or);
        ConditionBuilder(self.0)
    }

    /// Apply the current conditions.
    pub fn should(mut self) -> StructPredicateBuilder {
        self.0.conditions.push_front(ConditionToken::Should);
        PredicateBuilder(self.0)
    }
}

impl StructPredicateBuilder {
    /// Predicate matching structs with the given name.
    pub fn have_simple_name(mut self, name: &str) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::HaveSimpleName(name.to_owned()),
            ));
        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching public structs.
    pub fn be_public(mut self) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::BePublic));
        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching private structs.
    pub fn be_private(mut self) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::BePrivate));
        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching struct implementing the given trait.
    pub fn implement(mut self, trait_name: &str) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::Implement(trait_name.to_owned()),
            ));

        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching struct which derives the given trait.
    pub fn derive(mut self, trait_name: &str) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::Derive(
                trait_name.to_owned(),
            )));

        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching struct which derives OR implement the given trait.
    pub fn implement_or_derive(mut self, trait_name: &str) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::ImplementOrDerive(trait_name.to_owned()),
            ));

        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching struct which no public fields.
    pub fn only_have_private_fields(mut self) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::OnlyHavePrivateFields,
            ));

        PredicateConjunctionBuilder(self.0)
    }

    /// Predicate matching struct without restricted visibility fields
    pub fn only_have_public_fields(mut self) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::OnlyHavePublicFields,
            ));

        PredicateConjunctionBuilder(self.0)
    }
}

impl StructPredicateConjunctionBuilder {
    /// Combine two predicate with`And` conjunction.
    pub fn and_should(mut self) -> StructPredicateBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::Conjunction(AssertionConjunction::AndShould));
        PredicateBuilder(self.0)
    }

    /// Combine two predicate with the `Or` conjunction.
    pub fn or_should(mut self) -> StructPredicateBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::Conjunction(AssertionConjunction::OrShould));
        PredicateBuilder(self.0)
    }
}

#[cfg(test)]
mod test {
    use crate::rule::structs::ConditionToken;
    use crate::rule::structs::{AssertionConjunction, AssertionToken, SimpleAssertions, Structs};
    use crate::rule::ArchRuleBuilder;
    use crate::Filters;
    use speculoos::prelude::*;

    #[test]
    fn should_build_arch_rule_for_struct() {
        let rule = Structs::that(Filters::default())
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
            .be_private()
            .or_should()
            .only_have_private_fields()
            .or_should()
            .only_have_public_fields();

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
                AssertionToken::SimpleAssertion(SimpleAssertions::OnlyHavePublicFields),
                AssertionToken::Conjunction(AssertionConjunction::OrShould),
                AssertionToken::SimpleAssertion(SimpleAssertions::OnlyHavePrivateFields),
                AssertionToken::Conjunction(AssertionConjunction::OrShould),
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
