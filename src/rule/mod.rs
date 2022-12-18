use crate::assertion_result::AssertionResult;
use crate::Filters;
use std::collections::VecDeque;
use std::fmt::Debug;

pub mod enums;
pub mod impl_block;
pub mod modules;
pub mod pattern;
pub mod structs;

#[derive(Debug)]
pub struct ArchRule<C: Condition + Debug, A: Assertion + Debug + Clone, S: Subject> {
    pub(crate) conditions: VecDeque<C>,
    pub(crate) assertions: VecDeque<A>,
    pub(crate) filters: Filters<'static>,
    pub(crate) subject: S,
    pub(crate) assertion_results: AssertionResult,
}

/// The subject of an [`ArchRule`], `archunit-rs` load your crate Ast once and expose it via:
/// [`ModuleMatches`], [`StructMatches`], etc.*
/// If you need to extend the existing [`ArchRules`] for those subjects, you can wrap them in a struct
/// and provide your custom implementation:
/// ```
pub trait Subject: Default {
    fn init(filters: &Filters<'static>) -> Self;
}

/// [`Condition`] are used to filter matching [`Subjects`].
/// You can write your own custom condition to create new rules.
///
/// **Example:**
///
/// Let's say your crate library expose some structs that are meant to be de/serializable.
/// You could add some custom condition like so:
///
/// ```rust
/// use archunit_rs::rule::Condition;
/// #[derive(Debug, PartialEq)]
/// pub enum CustomStructCondition {
///     ShouldBeSerializable,
///     ShouldBeDeserializable,
/// }
///
/// impl Condition for CustomStructCondition {}
/// ```
pub trait Condition: Debug + PartialEq {}

/// [`Assertion`] are used to filter matching [`Subjects`]
pub trait Assertion: Debug + PartialEq + Clone {}

pub trait CheckRule<C: Condition, A: Assertion, S: Subject, T: assertable::Assertable<C, A, S>>:
    Sized
{
    fn check(self) {
        let mut rule = self.get_rule();

        // If there are no condition we are matching on all items
        if rule.has_conditions() {
            rule.apply_conditions();
        }

        let success = rule.apply_assertions();
        if !success {
            let result = rule.assertion_results();
            panic!("{result}")
        }
    }

    fn get_rule(self) -> T;
}

pub(super) mod assertable {
    use crate::assertion_result::AssertionResult;
    use crate::rule::{Assertion, Condition, Subject};

    pub trait Assertable<C: Condition, A: Assertion, S: Subject> {
        fn apply_conditions(&mut self);
        fn apply_assertions(&mut self) -> bool;
        fn assertion_results(&self) -> &AssertionResult;
        fn has_conditions(&self) -> bool;
    }
}

impl<C, A, S> ArchRule<C, A, S>
where
    C: Condition,
    A: Assertion,
    S: Subject,
{
    fn new(filters: Filters<'static>) -> Self {
        ArchRule {
            conditions: VecDeque::new(),
            assertions: VecDeque::new(),
            filters,
            subject: S::default(),
            assertion_results: AssertionResult::new(),
        }
    }

    fn init_subject(&self) -> S {
        S::init(&self.filters)
    }
}

pub trait ArchRuleBuilder<C: Condition, P: Assertion, S: Subject>: Sized {
    /// Builder function for arch rule assertions, see [`ConditionBuilder`].
    fn that(filters: Filters<'static>) -> ConditionBuilder<C, P, S> {
        ConditionBuilder(ArchRule::<C, P, S>::new(filters))
    }

    /// Match all and returns a [`PredicateBuilder`].
    fn all_should(filters: Filters<'static>) -> PredicateBuilder<C, P, S> {
        let mut rule = ArchRule::<C, P, S>::new(filters);
        rule.subject = rule.init_subject();
        PredicateBuilder(rule)
    }
}

#[derive(Debug)]
pub struct ConditionBuilder<C: Condition, P: Assertion, S: Subject>(ArchRule<C, P, S>);

#[derive(Debug)]
pub struct ConditionConjunctionBuilder<C: Condition, P: Assertion, S: Subject>(ArchRule<C, P, S>);

#[derive(Debug)]
pub struct PredicateBuilder<C: Condition, P: Assertion, S: Subject>(ArchRule<C, P, S>);

#[derive(Debug)]
pub struct DependencyPredicateConjunctionBuilder<C: Condition, P: Assertion, S: Subject>(
    ArchRule<C, P, S>,
);

#[derive(Debug)]
pub struct PredicateConjunctionBuilder<C: Condition, P: Assertion, S: Subject>(ArchRule<C, P, S>);
