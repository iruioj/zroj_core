mod context;
mod serde_attr;
mod ts_attr;

use context::{ContainerContext, FieldContext, ProvideDefault, VariantContext};
use quote::quote;
use syn::{
    AttrStyle, Attribute, Expr, Field, Fields, GenericParam, Generics, Item, ItemEnum, ItemStruct,
    Lit, Meta, MetaNameValue,
};

struct AttrList(Vec<Meta>);

impl syn::parse::Parse for AttrList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let metas = input.parse_terminated(Meta::parse, syn::Token![,])?;
        Ok(AttrList(metas.into_iter().collect()))
    }
}
fn parse_attrs(ident: &str, attrs: &[Attribute]) -> Vec<Meta> {
    let mut r = Vec::new();
    attrs
        .iter()
        .filter_map(|a| {
            if matches!(a.style, AttrStyle::Outer) {
                if let Meta::List(list) = &a.meta {
                    if list.path.is_ident(ident) {
                        return Some(&list.tokens);
                    }
                }
            }
            None
        })
        .for_each(|tokens| {
            let tokens = tokens.clone();
            let mut metas = syn::parse2::<AttrList>(tokens).expect("parse serde attr list");
            r.append(&mut metas.0);
        });
    r
}

/// 对于带有泛型的类型，我们要求所有类型参数都实现 TsType
fn gen_where_clause(generics: Generics) -> Option<proc_macro2::TokenStream> {
    generics
        .params
        .iter()
        .filter_map(|e| match e {
            GenericParam::Type(x) => Some(x),
            _ => None,
        })
        // 要求所有 generic type 都实现 TypeDef
        .map(|e| {
            let ident = &e.ident;
            quote!( #ident: serde_ts_typing::TsType, )
        })
        .reduce(|mut acc, e| {
            acc.extend(e);
            acc
        })
        .map(|bounds| {
            if let Some(c) = generics.where_clause {
                let p = c.predicates;
                quote!(where #bounds #p )
            } else {
                quote!(where #bounds)
            }
        })
}

/// field 的 key 和 type context 与 type def
fn gen_field(
    ctxt: impl ProvideDefault<FieldContext>,
    field: Field,
) -> Option<(
    Option<String>,
    (proc_macro2::TokenStream, proc_macro2::TokenStream),
)> {
    let ctxt = ctxt.provide_default(FieldContext::from_attr(serde_attr::parse_field_attr(
        &field.attrs,
    )));
    if ctxt.is_skip() {
        return None;
    }
    if ctxt.flatten || ctxt.getter || ctxt.serialize_with || ctxt.with {
        unimplemented!()
    }
    let field_name = field
        .ident
        .map(|ident| ctxt.rename_field(ident.to_string()));
    let ty = field.ty;
    let tyctxt = quote!(<#ty as serde_ts_typing::TsType>::register_self_context(c););
    let tydef = quote!(<#ty as serde_ts_typing::TsType>::type_def());
    return Some((field_name, (tyctxt, tydef)));
}

// 返回 context 的构造代码和当前结构本身的类型构造代码
fn gen_fields(
    ctxt: impl ProvideDefault<FieldContext> + Copy,
    fields: Fields,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    match fields {
        syn::Fields::Named(fields) => {
            let mut tyctxts = Vec::new();
            let mut tydefs = Vec::new();
            for field in fields.named {
                if let Some((name, (tyctxt, tydef))) = gen_field(ctxt, field) {
                    let name = name.unwrap();
                    tyctxts.push(tyctxt);
                    tydefs.push((name, tydef));
                }
            }
            let tydef = tydefs.into_iter().fold(
                quote!(let mut r = std::collections::BTreeMap::new();),
                |mut a, (name, tydef)| {
                    a.extend(quote!(r.insert(#name.into(), #tydef);));
                    a
                },
            );
            let tydef = quote!(serde_ts_typing::TypeExpr::Record({ #tydef r }));
            let tyctxt = tyctxts
                .into_iter()
                .reduce(|mut a, b| {
                    a.extend(b);
                    a
                })
                .unwrap_or(quote!());

            (tyctxt, tydef)
        }
        syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
            // newtype struct
            if unnamed.len() == 1 {
                let field = unnamed.into_iter().next().unwrap();
                if let Some((name, (tyctxt, tydef))) = gen_field(ctxt, field) {
                    assert!(name.is_none());

                    (tyctxt, tydef)
                } else {
                    panic!("nothing to serialize")
                }
            }
            // tuple struct
            else {
                let mut tyctxts = Vec::new();
                let mut tydefs = Vec::new();
                for field in unnamed {
                    if let Some((name, (tyctxt, tydef))) = gen_field(ctxt, field) {
                        assert!(name.is_none());

                        tyctxts.push(tyctxt);
                        tydefs.push(tydef);
                    }
                }
                let tydef = tydefs
                    .into_iter()
                    .reduce(|mut a, b| {
                        a.extend(quote!(, #b));
                        a
                    })
                    .expect("nothing to serialize");
                let tydef = quote!(serde_ts_typing::TypeExpr::Tuple(vec![#tydef]));
                let tyctxt = tyctxts
                    .into_iter()
                    .reduce(|mut a, b| {
                        a.extend(b);
                        a
                    })
                    .unwrap_or(quote!());

                (tyctxt, tydef)
            }
        }
        // for unit struct
        syn::Fields::Unit => (
            quote!(),
            quote!(serde_ts_typing::TypeExpr::Value(
                serde_ts_typing::Value::Null
            )),
        ),
    }
}

fn derive_struct(input: ItemStruct) -> proc_macro2::TokenStream {
    let struct_name = input.ident;
    let generics = input.generics;
    let where_clause = gen_where_clause(generics.clone());
    let gparam = generics.params;

    let ctxt = ContainerContext::from_attr(serde_attr::parse_container_attr(&input.attrs));
    let ts_ctxt = ts_attr::ContainerContext::from_attr(ts_attr::parse_container_attr(&input.attrs));

    let (mut tyctxt, mut tydef) = gen_fields(&ctxt, input.fields);

    // 目前看来只有在结合了 tag 的时候有用
    let name = ctxt.rename(struct_name.to_string());
    // must be named fields
    if let Some(tag) = ctxt.tag() {
        tydef = quote!({
            serde_ts_typing::TypeExpr::Intersection(
                [
                    serde_ts_typing::TypeExpr::Record( [
                        (#tag.into(), serde_ts_typing::TypeExpr::Value(serde_ts_typing::Value::String(#name.into())))
                    ].into_iter().collect()),
                    #tydef
                ].into_iter().collect()
            )
        })
    }
    if ctxt.transparent || ctxt.remote || ctxt.into.is_some() {
        unimplemented!()
    }

    if !ts_ctxt.inline {
        let ty_name = ts_ctxt.name.unwrap_or(struct_name.to_string());
        tyctxt = {
            let mut head = quote!({
                let id = std::any::TypeId::of::<#struct_name>();
                if !c.contains(id) {
                    c.register(id, #ty_name.into(), #tydef);
                } else {
                    panic!("duplicate type")
                }
            });
            head.extend(tyctxt);
            head
        };
        tydef = quote!(
            serde_ts_typing::TypeExpr::Ident(std::any::TypeId::of::<#struct_name>(), #ty_name.into())
        );
    }

    quote! {
        impl<#gparam> serde_ts_typing::TsType for #struct_name<#gparam> #where_clause {
            fn register_context(c: &mut serde_ts_typing::Context) {
                #tyctxt
            }
            fn type_def() -> serde_ts_typing::TypeExpr {
                #tydef
            }
        }
    }
}

fn derive_enum(input: ItemEnum) -> proc_macro2::TokenStream {
    let enum_name = input.ident;
    let generics = input.generics;
    let where_clause = gen_where_clause(generics.clone());
    let gparam = generics.params;

    let ctxt = ContainerContext::from_attr(serde_attr::parse_container_attr(&input.attrs));
    let ts_ctxt = ts_attr::ContainerContext::from_attr(ts_attr::parse_container_attr(&input.attrs));

    let enum_ty_name = ts_ctxt.name.clone().unwrap_or(enum_name.to_string());
    let mut tyctxts = Vec::new();
    let mut tydefs = Vec::new();

    for var in input.variants {
        let var_ctxt = ctxt.provide_default(VariantContext::from_attr(
            serde_attr::parse_variant_attr(&var.attrs),
        ));

        if var_ctxt.is_skip() {
            continue;
        }
        if var_ctxt.serialize_with || var_ctxt.with || var.discriminant.is_some() {
            unimplemented!()
        }
        if ctxt.untagged() {
            unimplemented!()
        }

        let var_name = var_ctxt.rename_variant(var.ident.to_string());
        let is_unit = matches!(var.fields, Fields::Unit);
        let (tyctxt, mut tydef) = gen_fields(&var_ctxt, var.fields);
        tyctxts.push(tyctxt);
        if let (Some(tag), Some(ctag)) = (ctxt.tag(), ctxt.content_tag()) {
            // adjacently tagged
            tydef = quote!(
                serde_ts_typing::TypeExpr::Record([
                    (#tag.into(), serde_ts_typing::TypeExpr::Value(serde_ts_typing:: Value::String(#var_name.into()))),
                    (#ctag.into(), #tydef),
                ].into_iter().collect())
            )
        } else if let Some(tag) = ctxt.tag() {
            // internally tagged
            // 如果是 unnamed fields，serde 不会编译报错，而会在运行时 panic
            if is_unit {
                tydef = quote!(
                    serde_ts_typing::TypeExpr::Record([
                        (#tag.into(), serde_ts_typing::TypeExpr::Value(serde_ts_typing::Value::String(#var_name.into())))
                    ].into_iter().collect())
                )
            } else {
                // 对于 enum 里的 newtype(enum::unit) 可能会出问题
                tydef = quote!({
                    serde_ts_typing::TypeExpr::Intersection(
                        [
                            serde_ts_typing::TypeExpr::Record( [
                                (#tag.into(), serde_ts_typing::TypeExpr::Value(serde_ts_typing::Value::String(#var_name.into())))
                            ].into_iter().collect()),
                            #tydef
                        ].into_iter().collect()
                    )
                })
            }
        } else {
            // Externally tagged
            if is_unit {
                tydef = quote!(serde_ts_typing::TypeExpr::Value(serde_ts_typing::Value::String(#var_name.into())))
            } else {
                tydef = quote!(serde_ts_typing::TypeExpr::Record([(#var_name.into(), #tydef)].into_iter().collect()));
            }
        }

        if !ts_ctxt.variant_inline {
            let var_ty_name = enum_ty_name.clone() + &var.ident.to_string();
            tyctxts.push(quote!({
                c.register_variant(#var_ty_name.into(), #tydef);
            }));
            tydef = quote!(
                serde_ts_typing::TypeExpr::Ident(std::any::TypeId::of::<#enum_name>(), #var_ty_name.into())
            );
        }
        tydefs.push((var_name, tydef));
    }
    let mut tyctxt = tyctxts
        .into_iter()
        .reduce(|mut a, b| {
            a.extend(b);
            a
        })
        .unwrap_or(quote!());
    let mut tydef = {
        let tydef = tydefs.into_iter().fold(
            quote!(let mut r = std::collections::BTreeSet::new();),
            |mut a, (_, tydef)| {
                a.extend(quote!( r.insert(#tydef); ));
                a
            },
        );
        quote!(serde_ts_typing::TypeExpr::Union({ #tydef r }))
    };

    if ctxt.remote || ctxt.into.is_some() {
        unimplemented!()
    }

    if ts_ctxt.inline && ts_ctxt.name.is_some() {
        panic!("ts(inline) can't be used with ts(name = \"...\")")
    }
    if !ts_ctxt.inline {
        tyctxt = {
            let mut head = quote!({
                let id = std::any::TypeId::of::<#enum_name>();
                if !c.contains(id) {
                    c.register(id, #enum_ty_name.into(), #tydef);
                } else {
                    panic!("duplicate type")
                }
            });
            head.extend(tyctxt);
            head
        };
        tydef = quote!(
            serde_ts_typing::TypeExpr::Ident(std::any::TypeId::of::<#enum_name>(), #enum_ty_name.into())
        );
    }

    quote! {
        impl<#gparam> serde_ts_typing::TsType for #enum_name<#gparam> #where_clause {
            fn register_context(c: &mut serde_ts_typing::Context) {
                #tyctxt
            }
            fn type_def() -> serde_ts_typing::TypeExpr {
                #tydef
            }
        }
    }
}

/// 为实现了 Serialize 的类型提供 typescript 类型生成
#[proc_macro_derive(TsType, attributes(ts))]
pub fn derive_ts_type(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: Item = syn::parse_macro_input!(item);
    if let Item::Struct(input) = input {
        derive_struct(input).into()
    } else if let Item::Enum(input) = input {
        derive_enum(input).into()
    } else {
        unimplemented!()
    }
}
