use std::fs::File;
use std::{
    env,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use syn::{File as SynFile, Ident, Item};

#[derive(Debug)]
pub(crate) struct ModuleFilePath(pub(crate) PathBuf);

#[derive(Debug)]
pub(crate) struct Ast(pub SynFile);

#[derive(Debug)]
pub struct ModuleAst {
    pub(crate) name: String,
    pub(crate) location: ModuleFilePath,
    pub(crate) ast: Ast,
    pub(crate) submodules: Vec<ModuleAst>,
}

impl ModuleFilePath {
    pub fn get_ast(&self) -> Ast {
        let mut file = File::open(&self.0).expect("Unable to open file");
        let mut src = String::new();
        file.read_to_string(&mut src).expect("Unable to read file");
        Ast(syn::parse_file(&src).expect("Unable to parse file"))
    }

    fn get_dir(&self) -> &Path {
        self.0
            .parent()
            .expect("Submodule path should have a parent")
    }

    pub fn crate_root() -> Self {
        let working_directory =
            env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
        let cargo_manifest_path = PathBuf::from_str(&working_directory)
            .expect("CARGO_MANIFEST_DIR must point to a valid path");

        let lib_path = cargo_manifest_path.join("src").join("lib.rs");

        if lib_path.exists() {
            Self(lib_path)
        } else {
            Self(cargo_manifest_path.join("src").join("main.rs"))
        }
    }
}

impl ModuleAst {
    pub(crate) fn load_crate_ast() -> ModuleAst {
        let location = ModuleFilePath::crate_root();
        let ast = location.get_ast();
        let name = env!("CARGO_CRATE_NAME", "'CARGO_CRATE_NAME' should be set").to_owned();
        let name = name.replace('-', "_");
        let mut crate_root = ModuleAst {
            name,
            location,
            ast,
            submodules: Vec::with_capacity(0),
        };

        crate_root.load_submodules();
        crate_root
    }

    fn get_submodule(&self, ident: &Ident) -> Self {
        let base_dir = self.location.get_dir();
        let file_module = base_dir.to_path_buf().join(format!("{ident}.rs"));
        let directory_module = base_dir.to_path_buf().join(format!("{ident}/mod.rs"));
        if file_module.exists() {
            let name = ident.to_string();
            let location = ModuleFilePath(file_module);
            let ast = location.get_ast();
            Self {
                name,
                location,
                ast,
                submodules: vec![],
            }
        } else if directory_module.exists() {
            let name = ident.to_string();
            let location = ModuleFilePath(directory_module);
            let ast = location.get_ast();
            ModuleAst {
                name,
                location,
                ast,
                submodules: vec![],
            }
        } else {
            panic!("no modules path found for modules {ident}")
        }
    }

    fn load_submodules(&mut self) {
        let submodules = self.ast.get_modules_ident();
        let mut submodules: Vec<ModuleAst> = submodules
            .iter()
            .map(|module_ident| self.get_submodule(module_ident))
            .collect();

        for module in submodules.iter_mut() {
            module.load_submodules();
        }

        self.submodules = submodules;
    }
}

impl Ast {
    fn get_modules_ident(&self) -> Vec<Ident> {
        self.0
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Mod(item_mod) => Some(item_mod),
                _ => None,
            })
            // filtering out file declared modules
            .filter(|module| module.content.is_none())
            .map(|module| module.ident.clone())
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::ast::parse::ModuleAst;
    use crate::ast::visitor::ModuleOrCrateRoot;
    use crate::ast::ItemPath;

    #[test]
    fn test() {
        let mut ast = ModuleAst::load_crate_ast();
        let tree = ast.visit_modules(ModuleOrCrateRoot::CrateRoot);
        let _tree = tree.to_tree(&ItemPath::empty(), None);
    }
}
