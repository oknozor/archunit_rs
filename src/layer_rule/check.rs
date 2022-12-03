use crate::ast::{module_tree, ModuleUse};
use crate::layer_rule::{LayerAssertion, LayeredArchitecture};
use crate::rule::modules::ModuleMatches;
use miette::ErrReport;
use miette::Result;

use crate::layer_rule::report::ForbiddenLayerAccess;

#[derive(Debug)]
struct LayerRule {
    layer_name: String,
    layer_path: String,
    layer_assertions: LayerAssertion,
}

impl LayeredArchitecture {
    pub fn check(self) -> Result<(), Vec<ErrReport>> {
        let mut architecture_rule_violation = vec![];
        let modules = module_tree();
        let rules: Vec<LayerRule> = self
            .layer_definitions
            .iter()
            .filter_map(|(layer_name, layer_path)| {
                self.layer_assertions
                    .get(layer_name)
                    .cloned()
                    .map(|assertion| LayerRule {
                        layer_name: layer_name.to_string(),
                        layer_path: layer_path.to_string(),
                        layer_assertions: assertion,
                    })
            })
            .collect();

        for rule in &rules {
            match &rule.layer_assertions {
                LayerAssertion::MayNotBeAccessedByAnyLayer => {
                    let _all_except_current_layer =
                        modules.module_that(|module| module.ident != rule.layer_path);
                    todo!()
                }
                LayerAssertion::MayOnlyBeAccessedByLayers(layers) => {
                    // Get all layer that are not allowed
                    let permitted_modules: Vec<&String> = rules
                        .iter()
                        .filter(|rule| layers.contains(&rule.layer_name))
                        .map(|rule| &rule.layer_path)
                        .collect();

                    let forbidden_layers: ModuleMatches = modules.module_that(|module| {
                        // Not resides in ast
                        !module.path.reside_in_any(permitted_modules.as_slice())
                            // not reside in rule module
                            && !module.path.reside_in(&rule.layer_path)
                    });

                    // Get each module tree for forbidden layers
                    for (_module_path, tree) in forbidden_layers.0 {
                        // Flatten dependencies for each forbidden layers
                        for (path, (_, deps)) in tree.flatten_deps().0 {
                            // Find forbidden access
                            let forbidden_dependencies: Vec<&ModuleUse> = deps
                                .iter()
                                .filter(|deps| deps.matching(&rule.layer_path.clone()))
                                .collect();

                            // Act
                            if !forbidden_dependencies.is_empty() {
                                for dep in forbidden_dependencies {
                                    println!("{}", dep.parts);
                                }
                                println!(
                                    "Forbidden access to layer {} by layer {}",
                                    rule.layer_name, path
                                );
                                // println!("{:?}", forbidden_dependencies);
                            }
                        }
                    }
                }
                LayerAssertion::MayOnlyBeAccessedByLayer(layer) => {
                    let layer_module_path = self
                        .layer_definitions
                        .get(layer)
                        .expect("layer should be defined");

                    let forbidden_layers: ModuleMatches = modules.module_that(|module| {
                        // Not resides in ast
                        !module.path.reside_in(layer_module_path)
                            // not reside in rule module
                            && !module.path.reside_in(&rule.layer_path)
                    });

                    // Get each module tree for forbidden layers
                    for (_, tree) in forbidden_layers.0 {
                        // Flatten dependencies for each forbidden layers
                        for (path, (real_path, deps)) in tree.flatten_deps().0 {
                            for usage in deps {
                                let rule_module_path = rule.layer_path.as_str();
                                if usage.starts_with(rule_module_path) {
                                    let error = ForbiddenLayerAccess::from_span_and_location(
                                        usage.span,
                                        real_path,
                                        rule.layer_name.to_owned(),
                                        rule_module_path.to_owned(),
                                        path.to_string(),
                                    );

                                    architecture_rule_violation.push(error.into())
                                }
                            }
                            // Find forbidden access
                            let forbidden_dependencies: Vec<&ModuleUse> = deps
                                .iter()
                                .filter(|deps| deps.matching(&rule.layer_path))
                                .collect();

                            // Act
                            if !forbidden_dependencies.is_empty() {
                                for dep in forbidden_dependencies {
                                    println!("{}", dep.parts);
                                }
                                println!(
                                    "Forbidden access to layer {} by layer {}",
                                    rule.layer_name, path
                                );
                                // println!("{:?}", forbidden_dependencies);
                            }
                        }
                    }
                }
            }
        }

        if architecture_rule_violation.is_empty() {
            Ok(())
        } else {
            Err(architecture_rule_violation)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::layer_rule::{layered_architecture, LayerAssertionBuilder, LayerDefinitionBuilder};

    #[test]
    fn test() {
        layered_architecture()
            .layer("Rule")
            .defined_by("archunit_rs::rule")
            .layer("Ast")
            .defined_by("archunit_rs::ast")
            .where_layer("Rule");
        // .may_only_be_accessed_by_layer("Ast")
        // .check()
        // .unwrap();
    }
}
