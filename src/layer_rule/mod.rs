use miette::ErrReport;
use std::collections::HashMap;

pub mod check;
pub mod report;

trait LayerDefinitionBuilder {
    type DefinitionBuilder;
    fn layer(self, layer: &str) -> Self::DefinitionBuilder;
}

trait LayerAssertionBuilder {
    type AssertionBuilder;
    fn where_layer(self, layer: &str) -> Self::AssertionBuilder;
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct LayeredArchitecture {
    // A named layer mapping to its actual module  path
    layer_definitions: HashMap<String, String>,
    // Layer names mapped to their respective assertion
    layer_assertions: HashMap<String, LayerAssertion>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayeredArchitectureBuilder {
    architecture: LayeredArchitecture,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayeredArchitectureDefinitionBuilder {
    layer: String,
    architecture: LayeredArchitecture,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayeredArchitectureDefinitionChainBuilder {
    architecture: LayeredArchitecture,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayerArchitectureAssertionBuilder {
    layer: String,
    architecture: LayeredArchitecture,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayerArchitectureAssertionChainBuilder {
    architecture: LayeredArchitecture,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LayerAssertion {
    MayNotBeAccessedByAnyLayer,
    MayOnlyBeAccessedByLayers(Vec<String>),
    MayOnlyBeAccessedByLayer(String),
}

pub fn layered_architecture() -> LayeredArchitectureBuilder {
    LayeredArchitectureBuilder {
        architecture: Default::default(),
    }
}

impl LayeredArchitectureBuilder {
    pub fn layer(self, layer: &str) -> LayeredArchitectureDefinitionBuilder {
        LayeredArchitectureDefinitionBuilder {
            layer: layer.to_owned(),
            architecture: LayeredArchitecture::default(),
        }
    }
}

impl LayeredArchitectureDefinitionBuilder {
    pub fn defined_by(mut self, module_path: &str) -> LayeredArchitectureDefinitionChainBuilder {
        self.architecture
            .layer_definitions
            .insert(self.layer, module_path.to_owned());
        LayeredArchitectureDefinitionChainBuilder {
            architecture: self.architecture,
        }
    }
}

impl LayerDefinitionBuilder for LayeredArchitectureDefinitionChainBuilder {
    type DefinitionBuilder = LayeredArchitectureDefinitionBuilder;

    fn layer(self, layer: &str) -> Self::DefinitionBuilder {
        LayeredArchitectureDefinitionBuilder {
            layer: layer.to_owned(),
            architecture: self.architecture,
        }
    }
}

impl LayerAssertionBuilder for LayeredArchitectureDefinitionChainBuilder {
    type AssertionBuilder = LayerArchitectureAssertionBuilder;

    fn where_layer(self, layer: &str) -> Self::AssertionBuilder {
        if self.architecture.layer_definitions.get(layer).is_none() {
            panic!("Undefined layer: '{layer}'")
        }

        LayerArchitectureAssertionBuilder {
            layer: layer.to_owned(),
            architecture: self.architecture,
        }
    }
}

impl LayerArchitectureAssertionBuilder {
    pub fn may_only_be_accessed_by_layer(
        mut self,
        layer: &str,
    ) -> LayerArchitectureAssertionChainBuilder {
        if self.architecture.layer_definitions.get(layer).is_none() {
            panic!("Undefined layer: '{layer}' in assertion `may_only_be_accessed_by_layer('{layer}')`")
        }
        self.architecture.layer_assertions.insert(
            self.layer,
            LayerAssertion::MayOnlyBeAccessedByLayer(layer.to_owned()),
        );
        LayerArchitectureAssertionChainBuilder {
            architecture: self.architecture,
        }
    }

    pub fn may_only_be_accessed_by_layers(
        mut self,
        layers: &[&str],
    ) -> LayerArchitectureAssertionChainBuilder {
        for layer in layers {
            if self
                .architecture
                .layer_definitions
                .get(&layer.to_string())
                .is_none()
            {
                panic!("Undefined layer: '{layer}' in assertion `may_only_be_accessed_by_layers('{layers:?}')`")
            }
        }

        let layers = layers.iter().map(|s| s.to_string()).collect();
        self.architecture.layer_assertions.insert(
            self.layer,
            LayerAssertion::MayOnlyBeAccessedByLayers(layers),
        );
        LayerArchitectureAssertionChainBuilder {
            architecture: self.architecture,
        }
    }

    pub fn may_not_be_accessed_by_any_layer(mut self) -> LayerArchitectureAssertionChainBuilder {
        self.architecture
            .layer_assertions
            .insert(self.layer, LayerAssertion::MayNotBeAccessedByAnyLayer);
        LayerArchitectureAssertionChainBuilder {
            architecture: self.architecture,
        }
    }
}

impl LayerAssertionBuilder for LayerArchitectureAssertionChainBuilder {
    type AssertionBuilder = LayerArchitectureAssertionBuilder;

    fn where_layer(self, layer: &str) -> LayerArchitectureAssertionBuilder {
        if self.architecture.layer_definitions.get(layer).is_none() {
            panic!("Undefined layer: '{layer}'")
        }

        LayerArchitectureAssertionBuilder {
            layer: layer.to_owned(),
            architecture: self.architecture,
        }
    }
}

impl LayerArchitectureAssertionChainBuilder {
    pub fn check(self) -> Result<(), ErrReport> {
        if let Err(errs) = self.architecture.check() {
            for err in errs {
                println!("{err:?}");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::layer_rule::{
        layered_architecture, LayerAssertion, LayerAssertionBuilder, LayerDefinitionBuilder,
    };
    use speculoos::prelude::*;

    #[test]
    fn test() {
        let architecture = layered_architecture()
            .layer("Rule")
            .defined_by("archunit_rs::rule")
            .layer("Ast")
            .defined_by("archunit_rs::ast")
            .where_layer("Ast")
            .may_not_be_accessed_by_any_layer()
            .where_layer("Rule")
            .may_only_be_accessed_by_layer("Ast");

        let layers = architecture.architecture.layer_definitions;
        let assertions = architecture.architecture.layer_assertions;

        assert_that!(layers.get("Rule"))
            .is_some()
            .is_equal_to(&"archunit_rs::rule".to_owned());

        assert_that!(layers.get("Ast"))
            .is_some()
            .is_equal_to(&"archunit_rs::ast".to_owned());

        assert_that!(assertions.get("Ast"))
            .is_some()
            .is_equal_to(&LayerAssertion::MayNotBeAccessedByAnyLayer);

        assert_that!(assertions.get("Rule"))
            .is_some()
            .is_equal_to(&LayerAssertion::MayOnlyBeAccessedByLayer("Ast".to_owned()));
    }
}
