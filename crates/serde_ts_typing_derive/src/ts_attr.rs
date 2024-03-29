use structural_macro_utils::AttrListVisitor;

use super::*;

pub enum ContainerAttr {
    /// `ts(name = "...")`
    ///
    /// change the name of this container's TypeScript type, default it's Rust name.
    Name(String),
    /// `ts(inline)`
    ///
    /// **do not** specify a type name for this container,
    /// be careful as it may lead to infinite recursion.
    ///
    /// Conflict with `name`
    Inline,

    /// `ts(variant_inline)`
    ///
    /// **do not** specify type names variants of enum, default it's container name + variant name.
    VariantInline,
}
pub fn parse_container_attr(attrs: AttrListVisitor<'_>) -> Vec<ContainerAttr> {
    attrs
        .get_list_by_ident("ts")
        .iter()
        .map(|meta| match meta {
            Meta::NameValue(item) => {
                if item.path.is_ident("name") {
                    if let Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &item.value
                    {
                        return ContainerAttr::Name(s.value());
                    }
                }
                panic!("invalid ts attr")
            }
            Meta::Path(item) => {
                if item.is_ident("inline") {
                    return ContainerAttr::Inline;
                }
                if item.is_ident("variant_inline") {
                    return ContainerAttr::VariantInline;
                }
                panic!("invalid ts attr")
            }
            Meta::List(_) => panic!("invalid ts attr"),
        })
        .collect()
}

#[derive(Default)]
pub struct ContainerContext {
    pub inline: bool,
    pub name: Option<String>,
    pub variant_inline: bool,
}

impl ContainerContext {
    pub fn from_attr(attrs: AttrListVisitor) -> ContainerContext {
        let attrs = crate::ts_attr::parse_container_attr(attrs);
        let mut r = ContainerContext::default();
        for attr in attrs {
            match attr {
                ContainerAttr::Name(s) => r.name = Some(s),
                ContainerAttr::Inline => r.inline = true,
                ContainerAttr::VariantInline => r.variant_inline = true,
            }
        }
        r
    }
}

pub enum FieldAttr {
    /// `ts(as_type = "...")`
    ///
    /// 指定该字段序列化后对应的 Rust 类型，用于配合 `serde(serialize_with)` 和 `serde(with)`
    As(String),
}
pub fn parse_field_attr(attrs: AttrListVisitor<'_>) -> Vec<FieldAttr> {
    attrs
        .get_list_by_ident("ts")
        .iter()
        .map(|meta| match meta {
            Meta::NameValue(item) => {
                if item.path.is_ident("as_type") {
                    if let Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &item.value
                    {
                        return FieldAttr::As(s.value());
                    }
                }
                panic!("invalid ts attr")
            }
            _ => panic!("invalid ts attr: {}", quote!(meta)),
        })
        .collect()
}

#[derive(Default)]
pub struct FieldContext {
    pub as_type: Option<String>,
}

impl FieldContext {
    pub fn from_attr(attrs: Vec<FieldAttr>) -> FieldContext {
        let mut r = FieldContext::default();
        for attr in attrs {
            match attr {
                FieldAttr::As(ty) => r.as_type = Some(ty),
            }
        }
        r
    }
}
