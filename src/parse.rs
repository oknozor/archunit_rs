use std::fs::File;
use std::{
    env,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

use syn::{File as SynFile, Ident, Item};

#[derive(Debug)]
pub(crate) struct ModulePath(PathBuf);

#[derive(Debug)]
pub(crate) struct Ast(SynFile);

#[derive(Debug)]
pub(crate) struct ModuleAst {
    pub(crate) name: String,
    pub(crate) location: ModulePath,
    pub(crate) ast: Ast,
    pub(crate) submodules: Vec<ModuleAst>,
}

impl ModulePath {
    fn get_ast(&self) -> Ast {
        let mut file = File::open(&self.0).expect("Unable to open file");
        let mut src = String::new();
        file.read_to_string(&mut src).expect("Unable to read file");
        Ast(syn::parse_file(&src).expect("Unable to parse file"))
    }

    fn get_dir(&self) -> &Path {
        self.0.parent().unwrap()
    }

    fn crate_root() -> Self {
        let working_directory = env::var("CARGO_MANIFEST_DIR").unwrap();
        let lib_path = PathBuf::from_str(&working_directory)
            .unwrap()
            .join("src")
            .join("lib.rs");

        if lib_path.exists() {
            Self(lib_path)
        } else {
            Self(
                PathBuf::from_str(&working_directory)
                    .unwrap()
                    .join("src")
                    .join("main.rs"),
            )
        }
    }
}

impl ModuleAst {
    fn get_submodule(&self, ident: &Ident) -> Self {
        let base_dir = self.location.get_dir();
        let file_module = base_dir.to_path_buf().join(format!("{ident}.rs"));
        let directory_module = base_dir.to_path_buf().join(format!("{ident}/mod.rs"));
        if file_module.exists() {
            let name = ident.to_string();
            let location = ModulePath(file_module);
            let ast = location.get_ast();
            Self {
                name,
                location,
                ast,
                submodules: vec![],
            }
        } else if directory_module.exists() {
            let name = ident.to_string();
            let location = ModulePath(directory_module);
            let ast = location.get_ast();
            ModuleAst {
                name,
                location,
                ast,
                submodules: vec![],
            }
        } else {
            panic!("no module path found for module {ident}")
        }
    }

    pub fn load_crate_root() -> ModuleAst {
        let location = ModulePath::crate_root();
        let ast = location.get_ast();
        let mut crate_root = ModuleAst {
            // Fixme: should be named after the crate name
            name: "crate_root".to_string(),
            location,
            ast,
            submodules: Vec::with_capacity(0),
        };

        crate_root.load_submodules();
        crate_root
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
