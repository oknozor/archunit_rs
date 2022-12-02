use std::fmt;
use std::fmt::Formatter;

use crate::ast::enums::Enum;
use impl_blocks::Impl;
use once_cell::sync::OnceCell;
use structs::Struct;
use syn::{ItemUse, UseTree};

use crate::ast::parse::ModuleAst;
use crate::ast::visitor::{ModuleOrCrateRoot, SynModuleTree};
use crate::rule::pattern::PathPattern;

pub mod enums;
pub mod impl_blocks;
pub(crate) mod parse;
pub mod structs;
pub mod visitor;

pub fn module_tree() -> &'static ModuleTree {
    static MODULE_TREE: OnceCell<ModuleTree> = OnceCell::new();
    MODULE_TREE.get_or_init(ModuleTree::load)
}

#[derive(Debug)]
pub struct ModuleTree {
    pub dependencies: Vec<ModuleUse>,
    pub path: ItemPath,
    pub ident: String,
    pub visibility: Visibility,
    pub structs: Vec<Struct>,
    pub enums: Vec<Enum>,
    pub impl_blocks: Vec<Impl>,
    pub submodules: Vec<ModuleTree>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ItemPath {
    inner: String,
}

impl ItemPath {
    pub fn new(path: String) -> Self {
        Self { inner: path }
    }
    pub fn empty() -> Self {
        ItemPath {
            inner: "".to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    pub fn join<S: AsRef<str>>(&self, path: S) -> Self {
        let mut item_path = self.clone();

        if !item_path.inner.is_empty() {
            item_path.inner.push_str("::");
        }
        item_path.inner.push_str(path.as_ref());
        item_path
    }

    pub fn match_module_path(&self, pattern: &str) -> bool {
        PathPattern::from(pattern).matches_module_path(&self.inner)
    }

    pub fn match_struct_path(&self, pattern: &str) -> bool {
        PathPattern::from(pattern).matches_struct_path(&self.inner)
    }

    pub fn name(&self) -> &str {
        if let Some((_, name)) = self.inner.rsplit_once("::") {
            name
        } else {
            self.as_str()
        }
    }

    pub fn contains(&self, other: &str) -> bool {
        self.inner.contains(other)
    }
}

impl fmt::Display for ItemPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug)]
pub struct ModuleUse {
    vis: Visibility,
    pub parts: String,
}

impl From<&ItemUse> for ModuleUse {
    fn from(item_use: &ItemUse) -> Self {
        let vis = Visibility::from_syn(&item_use.vis);
        let mut parts = String::new();
        if let UseTree::Path(path) = &item_use.tree {
            parts.push_str(&path.ident.to_string());
            let mut tree = &*path.tree;
            while let UseTree::Path(path) = tree {
                parts.push_str("::");
                parts.push_str(&path.ident.to_string());
                tree = &*path.tree;
            }
        }

        ModuleUse { vis, parts }
    }
}

impl ModuleTree {
    pub fn load() -> Self {
        let mut ast = ModuleAst::load_crate_ast();
        let syn_tree = ast.visit_modules(ModuleOrCrateRoot::CrateRoot);
        syn_tree.to_tree(&ItemPath::empty())
    }

    pub(crate) fn is_public(&self) -> bool {
        self.visibility == Visibility::Public
    }

    pub(crate) fn path_match(&self, pattern: &str) -> bool {
        self.path.match_module_path(pattern)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Visibility {
    Public,
    Crate,
    Restricted,
    Inherited,
}

impl Visibility {
    fn from_syn(vis: &syn::Visibility) -> Self {
        match vis {
            syn::Visibility::Public(_) => Visibility::Public,
            syn::Visibility::Crate(_) => Visibility::Crate,
            syn::Visibility::Restricted(_) => Visibility::Restricted,
            //TODO :  This is not correct : see : https://doc.rust-lang.org/reference/visibility-and-privacy.html
            // For now lets make this restricted, we'll fix this later
            syn::Visibility::Inherited => Visibility::Restricted,
        }
    }
}

impl SynModuleTree<'_> {
    pub(crate) fn to_tree(&self, path: &ItemPath) -> ModuleTree {
        let visibility = Visibility::from_syn(&self.module.vis());

        let dependencies = self.module.deps();
        let ident = self.module.ident().to_string();
        let path = path.join(ident.as_str());
        let structs = self.module.structs(&path);
        let enums = self.module.enums(&path);
        let impl_blocks = self.module.impls(&path);

        let submodules = self
            .submodules
            .iter()
            .map(|syn_module| syn_module.to_tree(&path))
            .collect();

        ModuleTree {
            dependencies,
            path,
            ident,
            visibility,
            structs,
            enums,
            impl_blocks,
            submodules,
        }
    }
}
