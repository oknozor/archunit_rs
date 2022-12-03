use crate::ast::{CodeSpan, ItemPath, Visibility};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::{ItemStruct, Meta, NestedMeta};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Struct {
    pub span: CodeSpan,
    pub real_path: PathBuf,
    // parse me with quote to handle generics
    pub ident: String,
    pub derives: Vec<String>,
    pub visibility: Visibility,
    pub fields: Vec<Field>,
    pub path: ItemPath,
}

impl Struct {
    pub fn from_syn(struct_: &ItemStruct, path: &ItemPath, real_path: &Path) -> Self {
        let ident = struct_.ident.to_string();
        let path = path.join(&ident);
        let derives = struct_
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

        let fields = struct_.fields.iter().map(Field::from).collect();
        let span = struct_.ident.span().into();
        Self {
            span,
            real_path: real_path.to_path_buf(),
            ident,
            derives,
            visibility: Visibility::from_syn(&struct_.vis),
            fields,
            path,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Field {
    pub visibility: Visibility,
    pub name: Option<String>,
    pub span: CodeSpan,
    pub type_: String,
}

impl From<&syn::Field> for Field {
    fn from(field: &syn::Field) -> Self {
        Self {
            visibility: Visibility::from_syn(&field.vis),
            name: field.ident.as_ref().map(|ident| ident.to_string()),
            span: field.span().into(),
            type_: format!("{:?}", field.ty),
        }
    }
}

impl Struct {
    pub fn all(&self) -> bool {
        true
    }

    pub fn is_public(&self) -> bool {
        self.visibility == Visibility::Public
    }

    pub fn path_match(&self, pattern: &str) -> bool {
        self.path.match_struct_path(pattern)
    }

    pub fn derives(&self, trait_: &str) -> bool {
        self.derives.contains(&trait_.to_owned())
    }

    pub fn has_non_public_field(&self) -> bool {
        let has_non_public_field = self
            .fields
            .iter()
            .any(|field| !matches!(field.visibility, Visibility::Public));

        has_non_public_field
    }
}
