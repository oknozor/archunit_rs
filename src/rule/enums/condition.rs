use crate::ast::ModuleTree;
use crate::rule::enums::EnumMatches;
use crate::ExludeModules;
use std::collections::HashSet;

impl ModuleTree {
    pub(crate) fn flatten_enums(&'static self, filters: &ExludeModules<'static>) -> EnumMatches {
        let mut enums = HashSet::new();

        self.enums.iter().for_each(|enum_| {
            enums.insert(enum_);
        });

        self.submodules
            .iter()
            .filter(filters.filter())
            .flat_map(|sub| sub.flatten(filters).0)
            .for_each(|(_, module)| enums.extend(module.flatten_enums(filters).0));

        EnumMatches(enums)
    }
}

#[cfg(test)]
mod condition_test {
    use crate::ast::module_tree;
    use crate::ExludeModules;
    use speculoos::prelude::*;

    #[test]
    fn should_filter_enums() {
        let all = module_tree().flatten_enums(&ExludeModules::default());
        let matches = all.enums_that(|enum_| enum_.ident == "AssertionToken");
        assert_that!(matches.0).has_length(3);

        let matches = all.enums_that(|e| e.is_public());
        assert_that!(matches.0).is_not_empty();

        let matches = all.enums_that(|e| !e.is_public());
        assert_that!(matches.0).is_not_empty();
    }
}
