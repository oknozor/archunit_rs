use crate::ast::enums::Enum;
use crate::ast::impl_blocks::Impl;
use crate::ast::parse::ModuleAst;
use crate::ast::structs::Struct;
use crate::ast::{ItemPath, ModuleUse};
use std::env;
use syn::__private::Span;
use syn::visit::Visit;
use syn::{visit, File, Ident, Item, ItemMod, VisPublic};

impl ModuleAst {
    pub fn visit_modules<'ast>(
        &'ast mut self,
        module_ident: ModuleOrCrateRoot<'ast>,
    ) -> SynModuleTree<'ast> {
        let mut visitor = FileVisitor::default();

        let mut module = SynModuleTree {
            module: ModuleOrFile::SynFile(module_ident, &self.ast.0),
            submodules: vec![],
        };

        visitor.visit_file(&self.ast.0);

        let modules: Vec<SynModuleTree> = visitor
            .inner_modules
            .iter()
            .map(|module| SynModuleTree {
                module: ModuleOrFile::InnerModule(module),
                submodules: vec![],
            })
            .collect();

        module.submodules.extend(modules);

        let mut iter_mut = self.submodules.iter_mut();

        for item_mod in visitor.file_modules {
            let mod_name = item_mod.ident.to_string();
            let subtree_ast = iter_mut
                .find(|sub_module_ast| sub_module_ast.name == mod_name)
                .unwrap();
            let sub_tree = subtree_ast.visit_modules(ModuleOrCrateRoot::Module(item_mod));
            module.submodules.push(sub_tree);
        }

        module
    }
}

#[derive(Debug)]
pub struct SynModuleTree<'ast> {
    pub module: ModuleOrFile<'ast>,
    pub submodules: Vec<SynModuleTree<'ast>>,
}

#[derive(Debug)]
pub enum ModuleOrFile<'ast> {
    InnerModule(&'ast ItemMod),
    SynFile(ModuleOrCrateRoot<'ast>, &'ast File),
}

#[derive(Debug)]
pub enum ModuleOrCrateRoot<'ast> {
    CrateRoot,
    Module(&'ast ItemMod),
}

impl ModuleOrFile<'_> {
    pub fn deps(&self) -> Vec<ModuleUse> {
        match self {
            ModuleOrFile::InnerModule(module) => get_module_use_item(module),
            ModuleOrFile::SynFile(_module, file) => get_files_use_item(file),
        }
    }

    pub fn ident(&self) -> Ident {
        match self {
            ModuleOrFile::InnerModule(module) => module.ident.clone(),
            ModuleOrFile::SynFile(module, _) => module.ident(),
        }
    }

    pub fn vis(&self) -> syn::Visibility {
        match self {
            ModuleOrFile::InnerModule(module) => module.vis.clone(),
            ModuleOrFile::SynFile(module, _) => module.vis(),
        }
    }

    pub fn impls(&self, path: &ItemPath) -> Vec<Impl> {
        match self {
            ModuleOrFile::InnerModule(module) => get_module_impls(module, path),
            ModuleOrFile::SynFile(module, file) => {
                let mut structs = get_files_impls(file, path);

                match module {
                    ModuleOrCrateRoot::CrateRoot => {}
                    ModuleOrCrateRoot::Module(module) => {
                        structs.extend(get_module_impls(module, path))
                    }
                };

                structs
            }
        }
    }

    pub fn structs(&self, path: &ItemPath) -> Vec<Struct> {
        match self {
            ModuleOrFile::InnerModule(module) => get_module_structs(module, path),
            ModuleOrFile::SynFile(module, file) => {
                let mut structs = get_file_structs(file, path);

                match module {
                    ModuleOrCrateRoot::CrateRoot => {}
                    ModuleOrCrateRoot::Module(module) => {
                        structs.extend(get_module_structs(module, path))
                    }
                };

                structs
            }
        }
    }

    pub fn enums(&self, path: &ItemPath) -> Vec<Enum> {
        match self {
            ModuleOrFile::InnerModule(module) => get_module_enums(module, path),
            ModuleOrFile::SynFile(module, file) => {
                let mut structs = get_file_enums(file, path);

                match module {
                    ModuleOrCrateRoot::CrateRoot => {}
                    ModuleOrCrateRoot::Module(module) => {
                        structs.extend(get_module_enums(module, path))
                    }
                };

                structs
            }
        }
    }
}

impl ModuleOrCrateRoot<'_> {
    fn ident(&self) -> Ident {
        let name = env::var("CARGO_PKG_NAME").unwrap();
        let name = name.replace('-', "_");
        match self {
            ModuleOrCrateRoot::CrateRoot => Ident::new(name.as_str(), Span::mixed_site()),
            ModuleOrCrateRoot::Module(module) => module.ident.clone(),
        }
    }

    fn vis(&self) -> syn::Visibility {
        match self {
            ModuleOrCrateRoot::CrateRoot => syn::Visibility::Public(VisPublic {
                pub_token: Default::default(),
            }),
            ModuleOrCrateRoot::Module(module) => module.vis.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct FileVisitor<'ast> {
    inner_modules: Vec<&'ast ItemMod>,
    file_modules: Vec<&'ast ItemMod>,
}

impl<'ast> Visit<'ast> for FileVisitor<'ast> {
    fn visit_file(&mut self, node: &'ast File) {
        node.items
            .iter()
            .filter_map(|item| {
                if let Item::Mod(module) = item {
                    Some(module)
                } else {
                    None
                }
            })
            .for_each(|module| {
                if module.content.is_some() {
                    self.inner_modules.push(module);
                    self.visit_item_mod(module);
                } else {
                    self.file_modules.push(module)
                }
            })
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        visit::visit_item_mod(self, node)
    }
}

fn get_module_use_item(module: &ItemMod) -> Vec<ModuleUse> {
    module
        .content
        .as_ref()
        .map(|(_brace, content)| {
            content
                .iter()
                .filter_map(|item| match item {
                    Item::Use(u) => Some(ModuleUse::from(u)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn get_module_impls(module: &ItemMod, path: &ItemPath) -> Vec<Impl> {
    if let Some((_, items)) = &module.content {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Impl(imp) => Some(Impl::from((imp, path))),
                _ => None,
            })
            .collect()
    } else {
        vec![]
    }
}

fn get_module_structs(module: &ItemMod, path: &ItemPath) -> Vec<Struct> {
    if let Some((_, items)) = &module.content {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Struct(struct_) => Some(Struct::from((struct_, path))),
                _ => None,
            })
            .collect()
    } else {
        vec![]
    }
}

fn get_module_enums(module: &ItemMod, path: &ItemPath) -> Vec<Enum> {
    if let Some((_, items)) = &module.content {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Enum(enum_) => Some(Enum::from((enum_, path))),
                _ => None,
            })
            .collect()
    } else {
        vec![]
    }
}

fn get_files_impls(file: &File, path: &ItemPath) -> Vec<Impl> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(imp) => Some(Impl::from((imp, path))),
            _ => None,
        })
        .collect()
}

fn get_file_structs(file: &File, path: &ItemPath) -> Vec<Struct> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(struct_) => Some(Struct::from((struct_, path))),
            _ => None,
        })
        .collect()
}

fn get_file_enums(file: &File, path: &ItemPath) -> Vec<Enum> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Enum(enum_) => Some(Enum::from((enum_, path))),
            _ => None,
        })
        .collect()
}

fn get_files_use_item(file: &File) -> Vec<ModuleUse> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Use(use_) => Some(ModuleUse::from(use_)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod test {
    use crate::ast::parse::ModuleAst;
    use crate::ast::visitor::ModuleOrCrateRoot;

    #[test]
    fn should_visit_crate_modules() {
        ModuleAst::load_crate_ast();
        let mut ast = ModuleAst::load_crate_ast();
        let tree = ast.visit_modules(ModuleOrCrateRoot::CrateRoot);
        let _root = &tree.module;
    }
}
