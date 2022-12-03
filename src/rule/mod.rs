use crate::assertion_result::AssertionResult;
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
    pub(crate) subject: S,
    pub(crate) assertion_results: AssertionResult,
}

/// The subject of an [`ArchRule`], `archunit-rs` load your crate Ast once and expose it via:
/// [`ModuleMatches`], [`StructMatches`], etc.*
/// If you need to extend the existing [`ArchRules`] for those subjects, you can wrap them in a struct
/// and provide your custom implementation:
///
/// **Example:**
/// ```rust
/// use archunit_rs::rule::Subject;
/// use archunit_rs::rule::structs::StructMatches;
///
/// #[derive(Default)]
/// pub struct CustomStructMatches(StructMatches);
///
/// impl Subject for CustomStructMatches {}
/// ```
pub trait Subject: Default {}

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
        rule.apply_conditions();
        rule.apply_assertions();
        let result = rule.assertion_results();
        if !result.actual.is_empty() {
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
        fn apply_assertions(&mut self);
        fn assertion_results(&self) -> &AssertionResult;
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
            assertion_results: AssertionResult::new(),
        }
    }
}

pub trait ArchRuleBuilder<C: Condition, P: Assertion, S: Subject>: Sized {
    /// Builder function for arch rule assertions, see [`ConditionBuilder`].
    fn that() -> ConditionBuilder<C, P, S> {
        ConditionBuilder(ArchRule::<C, P, S>::new())
    }

    /// Match all and returns a [`PredicateBuilder`].
    fn all_should() -> PredicateBuilder<C, P, S> {
        PredicateBuilder(ArchRule::<C, P, S>::new())
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
