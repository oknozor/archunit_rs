use crate::ast::{ItemPath, Visibility};
use syn::{ItemEnum, Meta, NestedMeta};

#[derive(Debug, PartialEq, Hash)]
pub struct Enum {
    // parse me with quote to handle generics
    pub ident: String,
    pub derives: Vec<String>,
    pub visibility: Visibility,
    pub path: ItemPath,
}

impl From<(&ItemEnum, &ItemPath)> for Enum {
    fn from((enum_, path): (&ItemEnum, &ItemPath)) -> Self {
        let ident = enum_.ident.to_string();
        let path = path.join(&ident);
        let derives = enum_
            .attrs
            .iter()
            .filter(|attr| {
                attr.path
                    .get_ident()
                    .map(|ident| ident.to_string())
                    .map(|ident| ident == "derive")
                    .unwrap_or(false)
            })
            .filter_map(|attr| {
                let attr = attr.parse_meta().expect("failed to parse derive attribute");

                match attr {
                    Meta::List(list) => {
                        let derives: Vec<String> = list
                            .nested
                            .iter()
                            .filter_map(|nested| match nested {
                                NestedMeta::Meta(meta) => match meta {
                                    Meta::Path(path) => Some(
                                        path.segments
                                            .iter()
                                            .map(|segment| segment.ident.to_string()),
                                    ),
                                    _ => None,
                                },
                                NestedMeta::Lit(_) => None,
                            })
                            .flatten()
                            .collect();
                        Some(derives)
                    }
                    _ => None,
                }
            })
            .flatten()
            .collect();

        Self {
            ident,
            derives,
            visibility: Visibility::from_syn(&enum_.vis),
            path,
        }
    }
}

impl Eq for Enum {}

impl Enum {
    pub fn is_public(&self) -> bool {
        self.visibility == Visibility::Public
    }

    pub fn path_match(&self, pattern: &str) -> bool {
        self.path.match_struct_path(pattern)
    }

    pub fn derives(&self, trait_: &str) -> bool {
        self.derives.contains(&trait_.to_string())
    }
}
