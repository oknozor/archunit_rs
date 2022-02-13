use crate::assertion_result::AssertionResult;
use std::collections::VecDeque;
use std::fmt::Debug;

pub mod modules;
pub mod structs;

#[derive(Debug)]
pub struct ArchRule<C: Condition + Debug, A: Assertion + Debug, S: Subject> {
    pub(crate) conditions: VecDeque<C>,
    pub(crate) assertions: VecDeque<A>,
    pub(crate) subject: S,
    pub(crate) assertion_result: AssertionResult,
}

pub trait Subject: Default {}

pub trait Condition: Debug + PartialEq {}

pub trait Assertion: Debug + PartialEq {}

pub trait CheckRule<C: Condition, A: Assertion, S: Subject, T: assertable::Assertable<C, A, S>>:
    Sized
{
    fn check(self) {
        let mut rule = self.get_rule();
        rule.apply_conditions();
        rule.apply_assertions();
    }

    fn get_rule(self) -> T;
}

pub(super) mod assertable {
    use crate::rule::{Assertion, Condition, Subject};

    pub trait Assertable<C: Condition, A: Assertion, S: Subject> {
        fn apply_conditions(&mut self);
        fn apply_assertions(&mut self);
    }
}

impl<C, A, S> ArchRule<C, A, S>
where
    C: Condition,
    A: Assertion,
    S: Subject,
{
    fn new() -> Self {
        ArchRule {
            conditions: VecDeque::new(),
            assertions: VecDeque::new(),
            subject: S::default(),
            assertion_result: AssertionResult::new(),
        }
    }
}

pub trait ArchRuleBuilder<C: Condition, P: Assertion, S: Subject>: Sized {
    fn that() -> ConditionBuilder<C, P, S> {
        ConditionBuilder(ArchRule::<C, P, S>::new())
    }
}

pub struct ConditionBuilder<C: Condition, P: Assertion, S: Subject>(ArchRule<C, P, S>);

pub struct ConditionConjunctionBuilder<C: Condition, P: Assertion, S: Subject>(ArchRule<C, P, S>);

pub struct PredicateBuilder<C: Condition, P: Assertion, S: Subject>(ArchRule<C, P, S>);

pub struct DependencyPredicateConjunctionBuilder<C: Condition, P: Assertion, S: Subject>(
    ArchRule<C, P, S>,
);

pub struct PredicateConjunctionBuilder<C: Condition, P: Assertion, S: Subject>(ArchRule<C, P, S>);
