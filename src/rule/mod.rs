mod module;

pub trait Condition {}

pub trait Assertion {}

struct ArchOperatorBuilder<C: Condition, P: Assertion>(ArchRule<C, P>);

struct ArchPredicateBuilder<C: Condition, P: Assertion>(ArchRule<C, P>);

struct ArchConditionBuilder<C: Condition, P: Assertion>(ArchRule<C, P>);

pub struct ArchRule<C: Condition, A: Assertion> {
    pub(crate) conditions: Vec<C>,
    pub(crate) assertions: Vec<A>,
}

impl<C, A> Default for ArchRule<C, A>
where
    C: Condition,
    A: Assertion,
{
    fn default() -> Self {
        ArchRule {
            conditions: vec![],
            assertions: vec![],
        }
    }
}
