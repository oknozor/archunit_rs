use crate::ast::impl_blocks::Impl;
use crate::ast::module_tree;
use crate::ModuleTree;
use once_cell::sync::OnceCell;
use std::collections::HashSet;

pub(crate) fn impl_matches() -> &'static ImplMatchesTODO {
    static INSTANCE: OnceCell<ImplMatchesTODO> = OnceCell::new();
    INSTANCE.get_or_init(|| module_tree().flatten_impls())
}

impl ModuleTree {
    pub(crate) fn flatten_impls(&'static self) -> ImplMatchesTODO {
        let mut impls = HashSet::new();

        self.impl_blocks.iter().for_each(|impl_block| {
            impls.insert(impl_block);
        });

        self.submodules
            .iter()
            .flat_map(|sub| sub.flatten().0)
            .for_each(|(_, module)| impls.extend(module.flatten_impls().0));

        ImplMatchesTODO(impls)
    }
}

#[derive(Debug, Default)]
pub struct ImplMatchesTODO(pub(crate) HashSet<&'static Impl>);

impl ImplMatchesTODO {
    pub fn impl_that<P>(&self, mut predicate: P) -> ImplMatchesTODO
    where
        P: FnMut(&Impl) -> bool,
    {
        let mut set = HashSet::new();
        self.0
            .iter()
            .copied()
            .filter(|imp| predicate(imp))
            .for_each(|imp| {
                set.insert(imp);
            });

        ImplMatchesTODO(set)
    }

    pub fn types(&self) -> Vec<&str> {
        self.0.iter().map(|imp| imp.self_ty.name()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
