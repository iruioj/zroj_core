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
    VariantInline
}
pub fn parse_container_attr(attrs: &[Attribute]) -> Vec<ContainerAttr> {
    parse_attrs("ts", attrs)
        .into_iter()
        .map(|meta| match meta {
            Meta::NameValue(item) => {
                if item.path.is_ident("name") {
                    if let Expr::Lit(syn::ExprLit { lit, .. }) = &item.value {
                        if let Lit::Str(s) = lit {
                            return ContainerAttr::Name(s.value());
                        }
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
    pub fn from_attr(attrs: Vec<ContainerAttr>) -> ContainerContext {
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

/*
pub enum FieldAttr {
    /// `ts(ignore_context)`
    ///
    /// ignore context of this field, often used for recursive struct
    IgnoreContext,
}
pub fn parse_field_attr(attrs: &[Attribute]) -> Vec<FieldAttr> {
    parse_attrs("ts", attrs)
        .into_iter()
        .map(|meta| match meta {
            Meta::NameValue(_) => panic!("invalid ts attr"),
            Meta::Path(item) => {
                if item.is_ident("ignore_context") {
                    return FieldAttr::IgnoreContext;
                }
                panic!("invalid ts attr")
            }
            Meta::List(_) => panic!("invalid ts attr"),
        })
        .collect()
}

#[derive(Default)]
pub struct FieldContext {
    pub ignore_context: bool,
}

impl FieldContext {
    pub fn from_attr(attrs: Vec<FieldAttr>) -> FieldContext {
        let mut r = FieldContext::default();
        for attr in attrs {
            match attr {
                FieldAttr::IgnoreContext => r.ignore_context = true,
            }
        }
        r
    }
}
*/