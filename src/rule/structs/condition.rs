use crate::ast::{module_tree, Struct};
use crate::ModuleTree;
use once_cell::sync::OnceCell;
use std::collections::HashSet;

pub(crate) fn struct_matches() -> &'static StructMatches {
    static INSTANCE: OnceCell<StructMatches> = OnceCell::new();
    INSTANCE.get_or_init(|| module_tree().flatten_structs())
}

#[derive(Debug, Default)]
pub struct StructMatches(pub(crate) HashSet<&'static Struct>);

impl StructMatches {
    pub fn structs_that<P>(&self, mut predicate: P) -> StructMatches
    where
        P: FnMut(&Struct) -> bool,
    {
        let mut set = HashSet::new();
        self.0
            .iter()
            .copied()
            .filter(|struct_| predicate(struct_))
            .for_each(|struct_| {
                set.insert(struct_);
            });

        StructMatches(set)
    }

    pub fn extends(&mut self, other: StructMatches) {
        self.0.extend(other.0)
    }
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
