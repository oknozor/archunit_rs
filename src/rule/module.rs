use crate::rule::{
    ArchConditionBuilder, ArchOperatorBuilder, ArchPredicateBuilder, ArchRule, Assertion, Condition,
};

enum ModuleCondition {
    AreDeclaredPublic,
    ResidesInAModule { module_regex: String },
    ArePublic,
    ArePrivate,
    HaveSimpleName { expression: String },
    And,
    Or,
}

enum ModuleAssertion {
    OnlyHaveDependencyModuleThat(ModulePredicateBuilder),
}

enum ModuleAssertionBuilder {
    OnlyHaveDependencyModuleThat,
}

enum ModulePredicateBuilder {
    ArePublic,
    ArePrivate,
    HaveSimpleName { expression: String },
}

impl Condition for ModuleCondition {}

impl Assertion for ModuleAssertion {}

pub struct Modules;

impl Modules {
    fn that() -> ArchConditionBuilder<ModuleCondition, ModuleAssertion>
    where
        Self: Sized,
    {
        ArchConditionBuilder(ArchRule::default())
    }
}

impl ArchConditionBuilder<ModuleCondition, ModuleAssertion> {
    fn reside_in_a_module(
        mut self,
        module: &str,
    ) -> ArchOperatorBuilder<ModuleCondition, ModuleAssertion> {
        self.0.conditions.push(ModuleCondition::ResidesInAModule {
            module_regex: module.to_string(),
        });
        ArchOperatorBuilder(self.0)
    }

    fn are_declared_public(mut self) -> ArchOperatorBuilder<ModuleCondition, ModuleAssertion> {
        self.0.conditions.push(ModuleCondition::AreDeclaredPublic);
        ArchOperatorBuilder(self.0)
    }
}

impl ArchOperatorBuilder<ModuleCondition, ModuleAssertion> {
    fn and(mut self) -> ArchConditionBuilder<ModuleCondition, ModuleAssertion> {
        self.0.conditions.push(ModuleCondition::And);
        ArchConditionBuilder(self.0)
    }

    fn or(mut self) -> ArchConditionBuilder<ModuleCondition, ModuleAssertion> {
        self.0.conditions.push(ModuleCondition::Or);
        ArchConditionBuilder(self.0)
    }

    fn should(self) -> ArchPredicateBuilder<ModuleCondition, ModuleAssertion> {
        ArchPredicateBuilder(self.0)
    }
}

impl ArchPredicateBuilder<ModuleCondition, ModuleAssertion> {
    pub fn only_have_dependency_modules_that(self) -> ArchModulePredicateBuilder {
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
                        ModulePredicateBuilder::ArePrivate,
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
                        ModulePredicateBuilder::ArePublic,
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
                        ModulePredicateBuilder::HaveSimpleName {
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
    use crate::rule::module::{ArchPredicateBuilder, Modules};
    use crate::rule::ArchRule;
    use crate::Structs;

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
