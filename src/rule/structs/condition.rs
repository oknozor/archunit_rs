use crate::rule::structs::StructMatches;
use crate::{ExludeModules, ModuleTree};
use std::collections::HashSet;

impl ModuleTree {
    pub(crate) fn flatten_structs(
        &'static self,
        filters: &ExludeModules<'static>,
    ) -> StructMatches {
        let mut structs = HashSet::new();

        self.structs.iter().for_each(|struct_| {
            structs.insert(struct_);
        });

        self.submodules
            .iter()
            .filter(filters.filter())
            .flat_map(|sub| sub.flatten(filters).0)
            .for_each(|(_, module)| structs.extend(module.flatten_structs(filters).0));

        StructMatches(structs)
    }
}

#[cfg(test)]
mod condition_test {
    use crate::ast::module_tree;
    use crate::ExludeModules;
    use speculoos::prelude::*;

    #[test]
    fn should_check_assertion() {
        let all = module_tree().flatten_structs(&ExludeModules::default());
        let matches = all.structs_that(|struct_| struct_.ident == "Ast");
        assert_that!(matches.0).has_length(1);

        let matches = all.structs_that(|s| s.is_public());
        assert_that!(matches.0).is_not_empty();

        let matches = all.structs_that(|s| !s.is_public());
        assert_that!(matches.0).is_not_empty();
    }
}
