// TODO :
//   - load every file in the crate
//   - implement the architecture layer assertions
//   - implement visibility assertion
//   - implement function/struct/enum/module matchers

use crate::parse::ModuleAst;
use once_cell::sync::OnceCell;

mod assertion;
pub mod parse;

use once_cell::sync::Lazy;
use std::marker::PhantomData;
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};
use syn::ItemStruct;

pub trait Condition {}
pub trait Assertion {}

pub struct ArchOperator<C: Condition, P: Assertion>(ArchRule<C, P>);
pub struct ArchPredicate<C: Condition, P: Assertion>(ArchRule<C, P>);
pub struct ArchConditionBuilder<C: Condition, P: Assertion>(ArchRule<C, P>);

pub struct ArchRule<C: Condition, P: Assertion> {
    conditions: Vec<C>,
    assertions: Vec<P>,
}

enum ModuleCondition {
    AreDeclaredPublic,
    ResidesInAModule { module_regex: String },
    ArePublic,
    ArePrivate,
    HaveSimpleName { expression: String },
    And,
    Or,
}
enum ModulePredicate {
    ArePublic,
    ArePrivate,
    HaveSimpleName { expression: String },
}

impl Condition for ModuleCondition {}

enum ModuleAssertion {
    OnlyHaveDependencyModuleThat(ModulePredicate),
}

enum ModuleAssertionBuilder {
    OnlyHaveDependencyModuleThat,
}

impl Assertion for ModuleAssertion {}

impl<C, P> ArchRule<C, P>
where
    C: Condition,
    P: Assertion,
{
    pub fn new() -> Self {
        ArchRule {
            conditions: vec![],
            assertions: vec![],
        }
    }
}

pub struct Modules;

impl Modules {
    fn that() -> ArchConditionBuilder<ModuleCondition, ModuleAssertion>
    where
        Self: Sized,
    {
        ArchConditionBuilder(ArchRule::new())
    }
}

impl ArchConditionBuilder<ModuleCondition, ModuleAssertion> {
    fn reside_in_a_module(
        mut self,
        module: &str,
    ) -> ArchOperator<ModuleCondition, ModuleAssertion> {
        self.0.conditions.push(ModuleCondition::ResidesInAModule {
            module_regex: module.to_string(),
        });
        ArchOperator(self.0)
    }

    fn are_declared_public(mut self) -> ArchOperator<ModuleCondition, ModuleAssertion> {
        self.0.conditions.push(ModuleCondition::AreDeclaredPublic);
        ArchOperator(self.0)
    }
}

impl ArchOperator<ModuleCondition, ModuleAssertion> {
    fn and(mut self) -> ArchConditionBuilder<ModuleCondition, ModuleAssertion> {
        self.0.conditions.push(ModuleCondition::And);
        ArchConditionBuilder(self.0)
    }

    fn or(mut self) -> ArchConditionBuilder<ModuleCondition, ModuleAssertion> {
        self.0.conditions.push(ModuleCondition::Or);
        ArchConditionBuilder(self.0)
    }

    fn should(self) -> ArchPredicate<ModuleCondition, ModuleAssertion> {
        ArchPredicate(self.0)
    }
}

impl ArchPredicate<ModuleCondition, ModuleAssertion> {
    pub fn only_have_dependency_modules_that(mut self) -> ArchModulePredicateBuilder {
        ArchModulePredicateBuilder {
            assertions: ModuleAssertionBuilder::OnlyHaveDependencyModuleThat,
            rule: self.0,
        }
    }
}

pub struct ArchModulePredicateBuilder {
    assertions: ModuleAssertionBuilder,
    rule: ArchRule<ModuleCondition, ModuleAssertion>,
}

impl ArchModulePredicateBuilder {
    fn are_private(mut self) -> ArchRule<ModuleCondition, ModuleAssertion> {
        match self.assertions {
            ModuleAssertionBuilder::OnlyHaveDependencyModuleThat => {
                self.rule
                    .assertions
                    .push(ModuleAssertion::OnlyHaveDependencyModuleThat(
                        ModulePredicate::ArePrivate,
                    ))
            }
        }

        self.rule
    }

    fn are_public(mut self) -> ArchRule<ModuleCondition, ModuleAssertion> {
        match self.assertions {
            ModuleAssertionBuilder::OnlyHaveDependencyModuleThat => {
                self.rule
                    .assertions
                    .push(ModuleAssertion::OnlyHaveDependencyModuleThat(
                        ModulePredicate::ArePublic,
                    ))
            }
        }

        self.rule
    }

    fn have_simple_name(mut self, expression: &str) -> ArchRule<ModuleCondition, ModuleAssertion> {
        match self.assertions {
            ModuleAssertionBuilder::OnlyHaveDependencyModuleThat => {
                self.rule
                    .assertions
                    .push(ModuleAssertion::OnlyHaveDependencyModuleThat(
                        ModulePredicate::HaveSimpleName {
                            expression: expression.to_string(),
                        },
                    ))
            }
        }

        self.rule
    }
}

#[cfg(test)]
mod test {
    use crate::{ArchPredicate, Modules};
    use crate::{ArchRule, Structs};

    #[test]
    fn public_api_construct() {
        let rule = Modules::that()
            .reside_in_a_module("foo::bar")
            .and()
            .are_declared_public()
            .should()
            .only_have_dependency_modules_that()
            .have_simple_name("baz");

        let rule = Modules::that()
            .are_declared_public()
            .should()
            .only_have_dependency_modules_that()
            .are_private();
    }
}

/*
ArchRule rule = Functions::that()
    .arePublic()
    .and()
    .are_declared_in_struct_that()
    .reside_in_a_module("::controller::")
    .should()
    .derive(Debug)
 */
// Structs::that()
//     .reside_in_a_module("foo::bar")
//     .should()
//     .be_public()
