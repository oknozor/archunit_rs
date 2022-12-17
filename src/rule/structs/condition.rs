use crate::ast::module_tree;
use crate::rule::structs::StructMatches;
use crate::{Filters, ModuleTree};
use std::collections::HashSet;

pub(crate) fn struct_matches(filters: &Filters<'static>) -> StructMatches {
    module_tree().flatten_structs(filters)
}

impl ModuleTree {
    pub(crate) fn flatten_structs(&'static self, filters: &Filters<'static>) -> StructMatches {
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
    use crate::rule::structs::condition::struct_matches;
    use crate::Filters;
    use speculoos::prelude::*;

    #[test]
    fn should_check_assertion() {
        let all = struct_matches(&Filters::default());
        let matches = all.structs_that(|struct_| struct_.ident == "Ast");
        assert_that!(matches.0).has_length(1);

        let matches = all.structs_that(|s| s.is_public());
        assert_that!(matches.0).is_not_empty();

        let matches = all.structs_that(|s| !s.is_public());
        assert_that!(matches.0).is_not_empty();
    }
}
