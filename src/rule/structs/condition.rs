use crate::ast::module_tree;
use crate::rule::enums::EnumMatches;
use crate::rule::structs::StructMatches;
use crate::ModuleTree;
use once_cell::sync::OnceCell;
use std::collections::HashSet;

pub(crate) fn struct_matches() -> &'static StructMatches {
    static MODULE_TREE: OnceCell<StructMatches> = OnceCell::new();
    MODULE_TREE.get_or_init(|| module_tree().flatten_structs())
}

pub(crate) fn enum_matches() -> &'static EnumMatches {
    static MODULE_TREE: OnceCell<EnumMatches> = OnceCell::new();
    MODULE_TREE.get_or_init(|| module_tree().flatten_enums())
}

impl ModuleTree {
    pub(crate) fn flatten_structs(&'static self) -> StructMatches {
        let mut structs = HashSet::new();

        self.structs.iter().for_each(|struct_| {
            structs.insert(struct_);
        });

        self.submodules
            .iter()
            .flat_map(|sub| sub.flatten().0)
            .for_each(|(_, module)| structs.extend(module.flatten_structs().0));

        StructMatches(structs)
    }

    pub(crate) fn flatten_enums(&'static self) -> EnumMatches {
        let mut enums = HashSet::new();

        self.enums.iter().for_each(|enum_| {
            enums.insert(enum_);
        });

        self.submodules
            .iter()
            .flat_map(|sub| sub.flatten().0)
            .for_each(|(_, module)| enums.extend(module.flatten_enums().0));

        EnumMatches(enums)
    }
}

#[cfg(test)]
mod condition_test {
    use crate::rule::structs::condition::struct_matches;

    #[test]
    fn should_check_assertion() {
        let all = struct_matches();
        let matches = all.structs_that(|struct_| struct_.ident == "Ast");
        println!("{:?}", matches);

        let matches = all.structs_that(|s| s.is_public());
        println!("{:?}", matches);

        let matches = all.structs_that(|s| !s.is_public());
        println!("{:?}", matches);
    }
}
