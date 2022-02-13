use crate::rule::structs::condition::StructMatches;
use crate::rule::{
    ArchRuleBuilder, Assertion, Condition, ConditionBuilder, ConditionConjunctionBuilder,
    PredicateBuilder, PredicateConjunctionBuilder, Subject,
};

mod check;
mod condition;

#[derive(Debug)]
pub struct Structs;

impl ArchRuleBuilder<ConditionToken, AssertionToken, StructMatches> for Structs {}

pub type StructConditionBuilder = ConditionBuilder<ConditionToken, AssertionToken, StructMatches>;
pub type StructConditionConjunctionBuilder =
    ConditionConjunctionBuilder<ConditionToken, AssertionToken, StructMatches>;
pub type StructPredicateBuilder = PredicateBuilder<ConditionToken, AssertionToken, StructMatches>;
pub type StructPredicateConjunctionBuilder =
    PredicateConjunctionBuilder<ConditionToken, AssertionToken, StructMatches>;

impl Condition for ConditionToken {}
impl Assertion for AssertionToken {}
impl Subject for StructMatches {}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum AssertionToken {
    SimpleAssertion(SimpleAssertions),
    Conjunction(AssertionConjunction),
}

#[derive(Debug, PartialEq)]
pub enum SimpleAssertions {
    BePublic,
    BePrivate,
    HaveSimpleName(String),
    Implement(String),
    Derive(String),
    OnlyHavePrivateFields,
    OnlyHavePublicFields,
}

#[derive(Debug, PartialEq)]
pub enum AssertionConjunction {
    AndShould,
    OrShould,
}

impl StructConditionBuilder {
    pub fn reside_in_a_module(mut self, module: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::ResidesInAModule(module.to_string()));
        ConditionConjunctionBuilder(self.0)
    }

    pub fn are_declared_public(mut self) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::AreDeclaredPublic);
        ConditionConjunctionBuilder(self.0)
    }

    pub fn are_declared_private(mut self) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::AreDeclaredPrivate);
        ConditionConjunctionBuilder(self.0)
    }

    pub fn have_simple_name(mut self, name: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::HaveSimpleName(name.to_string()));
        ConditionConjunctionBuilder(self.0)
    }

    pub fn derives(mut self, trait_name: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::Derives(trait_name.to_string()));
        ConditionConjunctionBuilder(self.0)
    }

    pub fn implement(mut self, trait_name: &str) -> StructConditionConjunctionBuilder {
        self.0
            .conditions
            .push_front(ConditionToken::Implement(trait_name.to_string()));
        ConditionConjunctionBuilder(self.0)
    }
}

impl StructConditionConjunctionBuilder {
    pub fn and(mut self) -> StructConditionBuilder {
        self.0.conditions.push_front(ConditionToken::And);
        ConditionBuilder(self.0)
    }

    pub fn or(mut self) -> StructConditionBuilder {
        self.0.conditions.push_front(ConditionToken::Or);
        ConditionBuilder(self.0)
    }

    pub fn should(mut self) -> StructPredicateBuilder {
        self.0.conditions.push_front(ConditionToken::Should);
        PredicateBuilder(self.0)
    }
}

impl StructPredicateBuilder {
    pub fn have_simple_name(mut self, name: &str) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::HaveSimpleName(name.to_string()),
            ));
        PredicateConjunctionBuilder(self.0)
    }

    pub fn be_public(mut self) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::BePublic));
        PredicateConjunctionBuilder(self.0)
    }

    pub fn be_private(mut self) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::BePrivate));
        PredicateConjunctionBuilder(self.0)
    }

    pub fn implement(mut self, trait_name: &str) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::Implement(trait_name.to_string()),
            ));

        PredicateConjunctionBuilder(self.0)
    }

    pub fn derive(mut self, trait_name: &str) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(SimpleAssertions::Derive(
                trait_name.to_string(),
            )));

        PredicateConjunctionBuilder(self.0)
    }

    pub fn only_have_private_fields(mut self) -> StructPredicateConjunctionBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::SimpleAssertion(
                SimpleAssertions::OnlyHavePrivateFields,
            ));

        PredicateConjunctionBuilder(self.0)
    }

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
    pub fn and_should(mut self) -> StructPredicateBuilder {
        self.0
            .assertions
            .push_front(AssertionToken::Conjunction(AssertionConjunction::AndShould));
        PredicateBuilder(self.0)
    }

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
    use speculoos::prelude::*;

    #[test]
    fn should_build_arch_rule_for_struct() {
        let rule = Structs::that()
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
                ConditionToken::HaveSimpleName("Name".to_string()),
                ConditionToken::And,
                ConditionToken::AreDeclaredPrivate,
                ConditionToken::And,
                ConditionToken::ResidesInAModule("::check".to_string()),
                ConditionToken::Or,
                ConditionToken::Implement("Display".to_string()),
                ConditionToken::And,
                ConditionToken::Derives("Debug".to_string()),
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
                    "Name".to_string(),
                )),
                AssertionToken::Conjunction(AssertionConjunction::AndShould),
                AssertionToken::SimpleAssertion(SimpleAssertions::BePublic),
                AssertionToken::Conjunction(AssertionConjunction::OrShould),
                AssertionToken::SimpleAssertion(SimpleAssertions::Implement("Name".to_string())),
            ]
            .iter(),
        )
    }
}
