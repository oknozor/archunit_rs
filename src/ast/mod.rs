use miette::SourceSpan;
use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;

use crate::ast::enums::Enum;
use impl_blocks::Impl;
use once_cell::sync::OnceCell;
use structs::Struct;
use syn::__private::Span;
use syn::spanned::Spanned;
use syn::{ItemMod, ItemUse, Meta, UseTree};

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
    pub span: Option<CodeSpan>,
    pub cfg_attr: Vec<String>,
    pub dependencies: Vec<ModuleUse>,
    pub real_path: PathBuf,
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
            inner: "".to_owned(),
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

    pub fn reside_in_any<S>(&self, allowed: &[S]) -> bool
    where
        S: AsRef<str>,
    {
        allowed
            .iter()
            .any(|allowed| self.inner.starts_with(allowed.as_ref()))
    }

    pub fn reside_in(&self, module: &str) -> bool {
        self.inner.starts_with(module)
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

fn get_item_mod_cfg(item: &ItemMod) -> Vec<String> {
    let mut cfg_attr = vec![];

    for attr in &item.attrs {
        let has_cfg = attr
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "cfg");

        if has_cfg {
            if let Ok(Meta::Path(path)) = attr.parse_args::<Meta>() {
                for attr_value in path.segments {
                    cfg_attr.push(attr_value.ident.to_string());
                }
            }
        }
    }
    cfg_attr
}

#[derive(Debug)]
pub struct ModuleUse {
    pub parts: String,
    pub span: CodeSpan,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CodeSpan {
    pub(crate) start: LineColumn,
    pub(crate) end: LineColumn,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl From<CodeSpan> for SourceSpan {
    fn from(value: CodeSpan) -> Self {
        (value.start.column, value.end.column - value.start.column).into()
    }
}

impl From<Span> for CodeSpan {
    fn from(span: Span) -> Self {
        Self {
            start: LineColumn {
                line: span.start().line,
                column: span.start().column,
            },
            end: LineColumn {
                line: span.end().line,
                column: span.end().column,
            },
        }
    }
}

impl From<&ItemUse> for ModuleUse {
    fn from(item_use: &ItemUse) -> Self {
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

        ModuleUse {
            parts,
            span: CodeSpan {
                start: LineColumn {
                    line: item_use.span().start().line,
                    column: item_use.span().start().column,
                },
                end: LineColumn {
                    line: item_use.span().end().line,
                    column: item_use.span().end().column,
                },
            },
        }
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
        let real_path = self.module.real_path();
        let cfg_attr = self.module.cfg_attr();
        let submodules = self
            .submodules
            .iter()
            .map(|syn_module| syn_module.to_tree(&path))
            .collect();
        let span = self.module.span();

        ModuleTree {
            span,
            cfg_attr,
            dependencies,
            real_path,
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

#[cfg(test)]
mod test {
    use crate::ast::ItemPath;
    use speculoos::prelude::*;

    #[test]
    fn should_reside_in_works() {
        let path = ItemPath {
            inner: "foo::bar::baz".to_owned(),
        };

        assert_that!(path.reside_in("foo")).is_true();
        assert_that!(path.reside_in("foo::bar")).is_true();
        assert_that!(path.reside_in("foo::bar:biz")).is_false();
        assert_that!(path.reside_in("bar::foo")).is_false();
    }

    #[test]
    fn should_reside_in_any() {
        let path = ItemPath {
            inner: "foo::bar::baz".to_owned(),
        };

        assert_that!(path.reside_in_any(&[
            &"foo".to_owned(),
            &"biz".to_owned(),
            &"bar".to_owned()
        ]))
        .is_true();
        assert_that!(path.reside_in_any(&[
            &"foo::bar".to_owned(),
            &"biz".to_owned(),
            &"bar".to_owned()
        ]))
        .is_true();
        assert_that!(path.reside_in_any(&[&"biz".to_owned(), &"bar".to_owned()])).is_false();
    }
}
