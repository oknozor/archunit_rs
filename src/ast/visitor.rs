use crate::ast::enums::Enum;
use crate::ast::impl_blocks::Impl;
use crate::ast::parse::ModuleAst;
use crate::ast::structs::Struct;
use crate::ast::{get_item_mod_cfg, CodeSpan, ItemPath, ModuleUse};
use std::env;
use std::path::{Path, PathBuf};
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
            module: ModuleOrFile::SynFile {
                module: module_ident,
                file: &self.ast.0,
                real_path: &self.location.0,
            },
            submodules: vec![],
        };

        visitor.visit_file(&self.ast.0);

        let modules: Vec<SynModuleTree> = visitor
            .inner_modules
            .iter()
            .map(|module| SynModuleTree {
                module: ModuleOrFile::InnerModule {
                    module,
                    real_path: &self.location.0,
                },
                submodules: vec![],
            })
            .collect();

        module.submodules.extend(modules);

        let mut iter_mut = self.submodules.iter_mut();

        for item_mod in visitor.file_modules {
            let mod_name = item_mod.ident.to_string();
            let subtree_ast = iter_mut
                .find(|sub_module_ast| sub_module_ast.name == mod_name)
                .expect("submodule exist");
            let sub_tree = subtree_ast.visit_modules(ModuleOrCrateRoot::Module {
                module: item_mod,
                real_path: subtree_ast.location.0.clone(),
            });
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
    InnerModule {
        module: &'ast ItemMod,
        real_path: &'ast PathBuf,
    },
    SynFile {
        module: ModuleOrCrateRoot<'ast>,
        file: &'ast File,
        real_path: &'ast PathBuf,
    },
}

#[derive(Debug)]
pub enum ModuleOrCrateRoot<'ast> {
    CrateRoot,
    Module {
        module: &'ast ItemMod,
        real_path: PathBuf,
    },
}

impl ModuleOrFile<'_> {
    pub fn real_path(&self) -> PathBuf {
        match self {
            ModuleOrFile::InnerModule { real_path, .. } => real_path.to_path_buf(),
            ModuleOrFile::SynFile { real_path, .. } => real_path.to_path_buf(),
        }
    }

    pub fn cfg_attr(&self) -> Vec<String> {
        match self {
            ModuleOrFile::InnerModule { module, .. } => get_item_mod_cfg(module),
            ModuleOrFile::SynFile { module, .. } => module.cfg_attr(),
        }
    }

    pub fn span(&self) -> Option<CodeSpan> {
        match self {
            ModuleOrFile::InnerModule { module, .. } => Some(module.ident.span().into()),
            ModuleOrFile::SynFile { module, .. } => module.span(),
        }
    }

    pub fn deps(&self) -> Vec<ModuleUse> {
        match self {
            ModuleOrFile::InnerModule { module, .. } => get_module_use_item(module),
            ModuleOrFile::SynFile { file, .. } => get_files_use_item(file),
        }
    }

    pub fn ident(&self) -> Ident {
        match self {
            ModuleOrFile::InnerModule { module, .. } => module.ident.clone(),
            ModuleOrFile::SynFile { module, .. } => module.ident(),
        }
    }

    pub fn vis(&self) -> syn::Visibility {
        match self {
            ModuleOrFile::InnerModule { module, .. } => module.vis.clone(),
            ModuleOrFile::SynFile { module, .. } => module.vis(),
        }
    }

    pub fn impls(&self, path: &ItemPath) -> Vec<Impl> {
        match self {
            ModuleOrFile::InnerModule { module, .. } => get_module_impls(module, path),
            ModuleOrFile::SynFile { module, file, .. } => {
                let mut structs = get_files_impls(file, path);

                match module {
                    ModuleOrCrateRoot::CrateRoot => {}
                    ModuleOrCrateRoot::Module { module, .. } => {
                        structs.extend(get_module_impls(module, path))
                    }
                };

                structs
            }
        }
    }

    pub fn structs(&self, path: &ItemPath) -> Vec<Struct> {
        match self {
            ModuleOrFile::InnerModule { module, real_path } => {
                get_module_structs(module, path, real_path)
            }
            ModuleOrFile::SynFile {
                module,
                file,
                real_path,
            } => {
                let mut structs = get_file_structs(file, path, real_path);

                match module {
                    ModuleOrCrateRoot::CrateRoot => {}
                    ModuleOrCrateRoot::Module { module, real_path } => {
                        structs.extend(get_module_structs(module, path, real_path))
                    }
                };

                structs
            }
        }
    }

    pub fn enums(&self, path: &ItemPath) -> Vec<Enum> {
        match self {
            ModuleOrFile::InnerModule {
                module, real_path, ..
            } => get_module_enums(module, path, real_path),
            ModuleOrFile::SynFile {
                module,
                file,
                real_path,
            } => {
                let mut structs = get_file_enums(file, path, real_path);

                match module {
                    ModuleOrCrateRoot::CrateRoot => {}
                    ModuleOrCrateRoot::Module { module, real_path } => {
                        structs.extend(get_module_enums(module, path, real_path))
                    }
                };

                structs
            }
        }
    }
}

impl ModuleOrCrateRoot<'_> {
    fn cfg_attr(&self) -> Vec<String> {
        match self {
            ModuleOrCrateRoot::CrateRoot => vec![],
            ModuleOrCrateRoot::Module { module, .. } => get_item_mod_cfg(module),
        }
    }

    fn span(&self) -> Option<CodeSpan> {
        match self {
            ModuleOrCrateRoot::CrateRoot => None,
            ModuleOrCrateRoot::Module { module, .. } => Some(module.ident.span().into()),
        }
    }

    fn ident(&self) -> Ident {
        let name = env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME must be set");
        let name = name.replace('-', "_");
        match self {
            ModuleOrCrateRoot::CrateRoot => Ident::new(name.as_str(), Span::mixed_site()),
            ModuleOrCrateRoot::Module { module, .. } => module.ident.clone(),
        }
    }

    fn vis(&self) -> syn::Visibility {
        match self {
            ModuleOrCrateRoot::CrateRoot => syn::Visibility::Public(VisPublic {
                pub_token: Default::default(),
            }),
            ModuleOrCrateRoot::Module { module, .. } => module.vis.clone(),
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

fn get_module_structs(module: &ItemMod, path: &ItemPath, real_path: &Path) -> Vec<Struct> {
    if let Some((_, items)) = &module.content {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Struct(struct_) => Some(Struct::from_syn(struct_, path, real_path)),
                _ => None,
            })
            .collect()
    } else {
        vec![]
    }
}

fn get_module_enums(module: &ItemMod, path: &ItemPath, real_path: &Path) -> Vec<Enum> {
    if let Some((_, items)) = &module.content {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Enum(enum_) => Some(Enum::from_syn(enum_, path, real_path)),
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

fn get_file_structs(file: &File, path: &ItemPath, real_path: &Path) -> Vec<Struct> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(struct_) => Some(Struct::from_syn(struct_, path, real_path)),
            _ => None,
        })
        .collect()
}

fn get_file_enums(file: &File, path: &ItemPath, real_path: &Path) -> Vec<Enum> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Enum(enum_) => Some(Enum::from_syn(enum_, path, real_path)),
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
