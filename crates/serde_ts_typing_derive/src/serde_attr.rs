//! 由于 serde 的行为会影响最终输出的数据格式，因此 TsType 必须对于所有的 serde attribute 都进行有效的处理
use structural_macro_utils::AttrListVisitor;

use super::*;

pub enum ContainerAttr {
    Rename(SeperableMeta<String>),
    RenameAll(SeperableMeta<RenameAllKind>),
    DenyUnknownFields,
    Tag(String),
    Content(String),
    Untagged,
    Bound(SeperableMeta<String>),
    /// default / default = "path"
    Default(Option<String>),
    Remote(String),
    Transparent,
    From(String),
    TryFrom(String),
    Into(String),
    Crate(String),
}

pub enum VariantAttr {
    Rename(SeperableMeta<String>),
    Alias(String),
    RenameAll(SeperableMeta<RenameAllKind>),
    Skip,
    SkipSerializing,
    SkipDeserializing,
    SerializeWith(String),
    DeserializeWith(String),
    With(String),
    Bound(SeperableMeta<String>),
    Borrow(Option<String>),
    Other,
}

pub enum FieldAttr {
    Rename(SeperableMeta<String>),
    Alias(String),
    /// default / default = "path"
    Default(Option<String>),
    Flatten,
    Skip,
    SkipSerializing,
    SkipDeserializing,
    SkipSerializingIf(String),
    SerializeWith(String),
    DeserializeWith(String),
    With(String),
    Bound(SeperableMeta<String>),
    Borrow(Option<String>),
    Getter(String),
}

pub enum SeperableMeta<T> {
    /// `[param] = "..."`
    Consistent(T),
    /// `[param](serialize = "...", deserialize = "...")`
    Seperate {
        serialize: Option<T>,
        deserialize: Option<T>,
    },
}

#[derive(Clone)]
pub enum RenameAllKind {
    /// lowercase
    Lowercase,
    /// UPPERCASE
    Uppercase,
    /// PascalCase
    PascalCase,
    /// camelCase
    CamelCase,
    /// snake_case
    SnakeCase,
    /// SCREAMING_SNAKE_CASE
    ScreamingSnakeCase,
    /// kebab-case
    KebabCase,
    /// SCREAMING-KEBAB-CASE
    ScreamingKebabCase,
}

impl RenameAllKind {
    pub fn apply_to_field(&self, field: &str) -> String {
        use RenameAllKind::*;
        match self {
            Lowercase | SnakeCase => field.to_owned(),
            Uppercase => field.to_ascii_uppercase(),
            PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in field.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            }
            CamelCase => {
                let pascal = PascalCase.apply_to_field(field);
                pascal[..1].to_ascii_lowercase() + &pascal[1..]
            }
            ScreamingSnakeCase => field.to_ascii_uppercase(),
            KebabCase => field.replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.apply_to_field(field).replace('_', "-"),
        }
    }
    pub fn apply_to_variant(&self, variant: &str) -> String {
        use RenameAllKind::*;
        match self {
            PascalCase => variant.to_owned(),
            Lowercase => variant.to_ascii_lowercase(),
            Uppercase => variant.to_ascii_uppercase(),
            CamelCase => variant[..1].to_ascii_lowercase() + &variant[1..],
            SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in variant.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            }
            ScreamingSnakeCase => SnakeCase.apply_to_variant(variant).to_ascii_uppercase(),
            KebabCase => SnakeCase.apply_to_variant(variant).replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase
                .apply_to_variant(variant)
                .replace('_', "-"),
        }
    }
}

impl From<String> for RenameAllKind {
    fn from(value: String) -> Self {
        match value.as_str() {
            "lowercase" => Self::Lowercase,
            "UPPERCASE" => Self::Uppercase,
            "PascalCase" => Self::PascalCase,
            "camelCase" => Self::CamelCase,
            "snake_case" => Self::SnakeCase,
            "SCREAMING_SNAKE_CASE" => Self::ScreamingSnakeCase,
            "kebab-case" => Self::KebabCase,
            "SCREAMING-KEBAB-CASE" => Self::ScreamingKebabCase,
            _ => panic!("invalid rename_all meta"),
        }
    }
}

macro_rules! str_value_ret {
    ($item:ident, $name:literal, $var:ident) => {
        if $item.path.is_ident($name) {
            if let Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) = &$item.value
            {
                return $var(s.value());
            }
        }
    };
}
macro_rules! sep_value_ret {
    ($item:ident, $name:literal, $var:ident) => {
        if $item.path.is_ident($name) {
            if let Expr::Lit(syn::ExprLit {
                lit: Lit::Str(s), ..
            }) = &$item.value
            {
                return $var(SeperableMeta::Consistent(s.value().into()));
            }
        }
    };
}

macro_rules! unit_value_ret {
    ($item:ident, $name:literal, $var:ident) => {
        if $item.is_ident($name) {
            return $var;
        }
    };
}

macro_rules! sep_value_sep_ret {
    ($item:ident, $name:literal, $var:ident) => {
        if $item.path.is_ident($name) {
            let metas = syn::parse2::<AttrList>($item.tokens.clone())
                .expect("parse serde attr list (sep_value_sep_ret)");
            let mut serialize = None;
            let mut deserialize = None;
            for meta in metas.0 {
                let Meta::NameValue(meta) = meta else {
                    panic!("invalid meta in serde attr")
                };
                if meta.path.is_ident("serialize") {
                    let Expr::Lit(syn::ExprLit { lit, .. }) = &meta.value else {
                        panic!("invalid meta in serde attr")
                    };
                    if let Lit::Str(s) = lit {
                        serialize = Some(s.value().into())
                    }
                }
                if meta.path.is_ident("deserialize") {
                    let Expr::Lit(syn::ExprLit { lit, .. }) = &meta.value else {
                        panic!("invalid meta in serde attr")
                    };
                    if let Lit::Str(s) = lit {
                        deserialize = Some(s.value().into())
                    }
                }
            }
            return $var(SeperableMeta::Seperate {
                serialize,
                deserialize,
            });
        }
    };
}

fn parse_container_attr_meta_name_value(item: &MetaNameValue) -> ContainerAttr {
    use ContainerAttr::*;

    str_value_ret!(item, "tag", Tag);
    str_value_ret!(item, "content", Content);
    str_value_ret!(item, "remote", Remote);
    str_value_ret!(item, "from", From);
    str_value_ret!(item, "try_from", TryFrom);
    str_value_ret!(item, "into", Into);
    str_value_ret!(item, "crate", Crate);

    sep_value_ret!(item, "rename", Rename);
    sep_value_ret!(item, "bound", Bound);
    sep_value_ret!(item, "rename_all", RenameAll);

    if item.path.is_ident("default") {
        if let Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) = &item.value
        {
            return Default(Some(s.value()));
        }
    }

    panic!("invalid serde attribute meta")
}

fn parse_container_attr_path(item: &syn::Path) -> ContainerAttr {
    use ContainerAttr::*;

    unit_value_ret!(item, "deny_unknown_fields", DenyUnknownFields);
    unit_value_ret!(item, "untagged", Untagged);
    unit_value_ret!(item, "transparent", Transparent);

    if item.is_ident("default") {
        return Default(None);
    }

    panic!("invalid serde attribute meta")
}

fn parse_container_attr_list(item: &syn::MetaList) -> ContainerAttr {
    use ContainerAttr::*;
    sep_value_sep_ret!(item, "rename", Rename);
    sep_value_sep_ret!(item, "rename_all", RenameAll);
    sep_value_sep_ret!(item, "bound", Bound);

    panic!("invalid serde attribute meta")
}
pub fn parse_container_attr(attrs: AttrListVisitor) -> Vec<ContainerAttr> {
    attrs
        .get_list_by_ident("serde")
        .iter()
        .map(|meta| match meta {
            Meta::NameValue(item) => parse_container_attr_meta_name_value(item),
            Meta::Path(item) => parse_container_attr_path(item),
            Meta::List(item) => parse_container_attr_list(item),
        })
        .collect()
}

fn parse_variant_attr_meta_name_value(item: &MetaNameValue) -> VariantAttr {
    use VariantAttr::*;

    sep_value_ret!(item, "rename", Rename);
    sep_value_ret!(item, "rename_all", RenameAll);
    sep_value_ret!(item, "bound", Bound);

    str_value_ret!(item, "alias", Alias);
    str_value_ret!(item, "serialize_with", SerializeWith);
    str_value_ret!(item, "deserialize_with", DeserializeWith);
    str_value_ret!(item, "with", With);

    if item.path.is_ident("borrow") {
        if let Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) = &item.value
        {
            return Borrow(Some(s.value()));
        }
    }

    panic!("invalid serde attribute meta")
}
fn parse_variant_attr_path(item: &syn::Path) -> VariantAttr {
    use VariantAttr::*;

    unit_value_ret!(item, "skip", Skip);
    unit_value_ret!(item, "skip_serializing", SkipSerializing);
    unit_value_ret!(item, "skip_deserializing", SkipDeserializing);
    unit_value_ret!(item, "other", Other);

    if item.is_ident("borrow") {
        return Borrow(None);
    }

    panic!("invalid serde attribute meta")
}
fn parse_variant_attr_list(item: &syn::MetaList) -> VariantAttr {
    use VariantAttr::*;

    sep_value_sep_ret!(item, "rename", Rename);
    sep_value_sep_ret!(item, "rename_all", RenameAll);
    sep_value_sep_ret!(item, "bound", Bound);

    panic!("invalid serde attribute meta")
}
pub fn parse_variant_attr(attrs: AttrListVisitor) -> Vec<VariantAttr> {
    attrs
        .get_list_by_ident("serde")
        .iter()
        .map(|meta| match meta {
            Meta::NameValue(item) => parse_variant_attr_meta_name_value(item),
            Meta::Path(item) => parse_variant_attr_path(item),
            Meta::List(item) => parse_variant_attr_list(item),
        })
        .collect()
}
fn parse_field_attr_meta_name_value(item: &MetaNameValue) -> FieldAttr {
    use FieldAttr::*;

    sep_value_ret!(item, "rename", Rename);
    sep_value_ret!(item, "bound", Bound);

    str_value_ret!(item, "getter", Getter);
    str_value_ret!(item, "alias", Alias);
    str_value_ret!(item, "skip_serializing_if", SkipSerializingIf);
    str_value_ret!(item, "serialize_with", SerializeWith);
    str_value_ret!(item, "deserialize_with", DeserializeWith);
    str_value_ret!(item, "with", With);

    if item.path.is_ident("default") {
        if let Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) = &item.value
        {
            return Default(Some(s.value()));
        }
    }
    if item.path.is_ident("borrow") {
        if let Expr::Lit(syn::ExprLit {
            lit: Lit::Str(s), ..
        }) = &item.value
        {
            return Borrow(Some(s.value()));
        }
    }

    panic!("invalid serde attribute meta")
}
fn parse_field_attr_path(item: &syn::Path) -> FieldAttr {
    use FieldAttr::*;

    unit_value_ret!(item, "flatten", Flatten);
    unit_value_ret!(item, "skip", Skip);
    unit_value_ret!(item, "skip_serializing", SkipSerializing);
    unit_value_ret!(item, "skip_deserializing", SkipDeserializing);

    if item.is_ident("default") {
        return Default(None);
    }
    if item.is_ident("borrow") {
        return Borrow(None);
    }

    panic!("invalid serde attribute meta")
}
fn parse_field_attr_list(item: &syn::MetaList) -> FieldAttr {
    use FieldAttr::*;

    sep_value_sep_ret!(item, "rename", Rename);
    sep_value_sep_ret!(item, "bound", Bound);

    panic!("invalid serde attribute meta")
}
pub fn parse_field_attr(attrs: AttrListVisitor) -> Vec<FieldAttr> {
    attrs
        .get_list_by_ident("serde")
        .iter()
        .map(|meta| match meta {
            Meta::NameValue(item) => parse_field_attr_meta_name_value(item),
            Meta::Path(item) => parse_field_attr_path(item),
            Meta::List(item) => parse_field_attr_list(item),
        })
        .collect()
}
