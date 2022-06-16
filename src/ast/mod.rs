use std::fmt;
use std::fmt::Formatter;

use once_cell::sync::OnceCell;
use syn::ItemStruct;

use crate::ast::parse::ModuleAst;
use crate::ast::visitor::{ModuleOrCrateRoot, SynModuleTree};

pub(crate) mod parse;
pub mod visitor;

pub fn module_tree() -> &'static ModuleTree {
    static INSTANCE: OnceCell<ModuleTree> = OnceCell::new();
    INSTANCE.get_or_init(ModuleTree::load)
}

#[derive(Debug, PartialEq, Hash)]
pub struct Struct {
    pub ident: String,
    pub derives: Vec<String>,
    pub visibility: Visibility,
    pub fields: Vec<Field>,
    pub path: ItemPath,
}

impl From<(&syn::ItemStruct, &ItemPath)> for Struct {
    fn from((struct_, path): (&ItemStruct, &ItemPath)) -> Self {
        let ident = struct_.ident.to_string();
        let path = path.join(&ident);
        let derives = struct_
            .attrs
            .iter()
            .filter_map(|attr| {
                attr.path
                    .get_ident()
                    .map(|ident| ident.to_string())
                    .filter(|ident| *ident == "derive")
            })
            .collect();

        let fields = struct_.fields.iter().map(Field::from).collect();

        Self {
            ident,
            derives,
            visibility: Visibility::from_syn(&struct_.vis),
            fields,
            path,
        }
    }
}

#[derive(Debug, PartialEq, Hash)]
pub struct Field {
    pub visibility: Visibility,
    pub name: String,
    pub type_: String,
}

impl From<&syn::Field> for Field {
    fn from(field: &syn::Field) -> Self {
        Self {
            visibility: Visibility::from_syn(&field.vis),
            // todo: replace unnamed field name with their index
            name: field
                .ident
                .as_ref()
                .map(|ident| ident.to_string())
                .unwrap_or_else(|| "-".to_string()),
            // todo: format this correctly
            type_: format!("{:?}", field.ty),
        }
    }
}

impl Eq for Struct {}

impl Struct {
    pub fn is_public(&self) -> bool {
        self.visibility == Visibility::Public
    }

    pub fn has_parent(&self, name: &str) -> bool {
        self.path.has_parent(name)
    }

    pub fn derives(&self, trait_: &str) -> bool {
        self.derives.contains(&trait_.to_string())
    }

    pub fn has_non_public_field(&self) -> bool {
        let has_non_public_field = self
            .fields
            .iter()
            .any(|field| !matches!(field.visibility, Visibility::Public));

        has_non_public_field
    }
}

#[derive(Debug)]
pub struct ModuleTree {
    pub path: ItemPath,
    pub ident: String,
    pub visibility: Visibility,
    pub structs: Vec<Struct>,
    pub submodules: Vec<ModuleTree>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ItemPath {
    inner: String,
}

impl ItemPath {
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

    pub fn has_parent(&self, parent_name: &str) -> bool {
        if let Some((path, _)) = self.inner.rsplit_once("::") {
            path.contains(parent_name)
        } else {
            false
        }
    }

    pub fn name(&self) -> &str {
        if let Some((_, name)) = self.inner.rsplit_once("::") {
            name
        } else {
            self.as_str()
        }
    }
}

impl fmt::Display for ItemPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
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

    pub(crate) fn has_parent(&self, parent_name: &str) -> bool {
        self.path.has_parent(parent_name)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
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

        let ident = self.module.ident().to_string();
        let path = path.join(ident.as_str());
        let structs = self.module.structs(&path);

        let submodules = self
            .submodules
            .iter()
            .map(|syn_module| syn_module.to_tree(&path))
            .collect();

        ModuleTree {
            path,
            ident,
            visibility,
            structs,
            submodules,
        }
    }
}
