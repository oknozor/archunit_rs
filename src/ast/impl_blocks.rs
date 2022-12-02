use crate::ast::ItemPath;
use std::fmt::Debug;
use syn::{ItemImpl, Type};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Impl {
    pub path: ItemPath,
    pub is_unsafe: bool,
    pub self_ty: ItemPath,
    pub trait_impl: Option<ItemPath>,
}

impl From<(&ItemImpl, &ItemPath)> for Impl {
    fn from((imp, path): (&ItemImpl, &ItemPath)) -> Self {
        let path = path.clone();
        let is_unsafe = imp.unsafety.is_some();

        let self_ty = &*imp.self_ty;
        let self_ty = match self_ty {
            Type::Path(p) => {
                let path = p
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect::<Vec<String>>()
                    .join("::");

                ItemPath::new(path)
            }
            _ => ItemPath::empty(),
        };

        let trait_impl = imp.trait_.as_ref().map(|(_, p, _)| {
            p.segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<String>>()
                .join("::")
        });

        let trait_impl = trait_impl.map(ItemPath::new);

        Self {
            is_unsafe,
            self_ty,
            path,
            trait_impl,
        }
    }
}
