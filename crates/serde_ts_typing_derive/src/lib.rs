mod context;
mod serde_attr;
mod ts_attr;

use context::{ContainerContext, FieldContext, ProvideDefault, VariantContext};
use quote::{format_ident, quote, ToTokens};
use structural_macro_utils::{
    concat_token_stream, EnumVisitor, FieldVisitor, FieldsVisitor, GenericsVisitor, StructVisitor,
};
use syn::{Expr, Fields, Item, ItemEnum, ItemStruct, Lit, Meta, MetaNameValue};

/// 对于带有泛型的类型，我们要求所有类型参数都实现 TsType
fn gen_where_clause(generics: GenericsVisitor) -> Option<proc_macro2::TokenStream> {
    generics
        .iter_type_params()
        .map(|e| {
            let ident = &e.ident;
            quote!( #ident: serde_ts_typing::TsType, )
        })
        .reduce(|mut acc, e| {
            acc.extend(e);
            acc
        })
        .map(|bounds| {
            if let Some(c) = &generics.0.where_clause {
                let p = &c.predicates;
                quote!(where #bounds #p )
            } else {
                quote!(where #bounds)
            }
        })
}

enum FieldKind {
    // 匿名字段
    Unnamed,
    // 带有字段名称
    Named(String),
    // 该字段的类型与 container 合并
    Flatten,
}

/// field 的 key 和 type context 与 type def
fn gen_field(
    ctxt: impl ProvideDefault<FieldContext>,
    field: FieldVisitor<'_>,
) -> Option<(
    FieldKind,
    (proc_macro2::TokenStream, proc_macro2::TokenStream),
)> {
    let ctxt = ctxt.provide_default(FieldContext::from_attr(field.attrs()));
    let ts_ctxt = ts_attr::FieldContext::from_attr(ts_attr::parse_field_attr(field.attrs()));
    if ctxt.is_skip() {
        return None;
    }
    if ctxt.getter {
        unimplemented!()
    }
    let field_kind = if ctxt.flatten {
        FieldKind::Flatten
    } else {
        field
            .ident()
            .map(|ident| ctxt.rename_field(ident.to_string()))
            .map_or(FieldKind::Unnamed, FieldKind::Named)
    };
    let ty = if ctxt.serialize_with || ctxt.with {
        let Some(ty_str) = ts_ctxt.as_type else {
            panic!("ts(as) must be used with serde(serialize_with) or serde(with)")
        };

        let ty_ident = format_ident!("{}", ty_str);
        ty_ident.into_token_stream()
    } else {
        field.ty().into_token_stream()
    };
    let tyctxt = quote!(<#ty as serde_ts_typing::TsType>::register_self_context(c););
    let tydef = quote!(<#ty as serde_ts_typing::TsType>::type_def());
    Some((field_kind, (tyctxt, tydef)))
}

// 返回 context 的构造代码 (return ()) 和当前结构本身的类型构造代码 (return TypeExpr)
fn gen_fields(
    ctxt: impl ProvideDefault<FieldContext> + Copy,
    fields: FieldsVisitor,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if fields.is_named() {
        let mut tyctxts = Vec::new();
        let mut tydefs = Vec::new();
        for field in fields.iter_fields() {
            if let Some((name, (tyctxt, tydef))) = gen_field(ctxt, field) {
                tyctxts.push(tyctxt);
                tydefs.push((name, tydef));
            }
        }
        let mut tydef = quote!(let mut r = serde_ts_typing::TypeExpr::new_struct(););
        for (name, fldef) in tydefs {
            if let FieldKind::Flatten = name {
                tydef.extend(quote!(r.struct_merge(#fldef);));
            } else if let FieldKind::Named(s) = name {
                tydef.extend(quote!(r.struct_insert(#s.into(), #fldef);));
            }
        }
        let tydef = quote!({ #tydef r });
        let tyctxt = concat_token_stream(tyctxts);

        (tyctxt, tydef)
    } else if fields.is_unnamed() {
        // newtype struct
        if let Some(field) = fields.the_only_field() {
            if let Some((name, (tyctxt, tydef))) = gen_field(ctxt, field) {
                assert!(matches!(name, FieldKind::Unnamed));

                (tyctxt, tydef)
            } else {
                panic!("nothing to serialize")
            }
        }
        // tuple struct
        else {
            let mut tyctxts = Vec::new();
            let mut tydefs = Vec::new();
            for field in fields.iter_fields() {
                if let Some((name, (tyctxt, tydef))) = gen_field(ctxt, field) {
                    assert!(matches!(name, FieldKind::Unnamed));

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
            let tyctxt = concat_token_stream(tyctxts);

            (tyctxt, tydef)
        }
    } else {
        // for unit struct
        (
            quote!(),
            quote!(serde_ts_typing::TypeExpr::Value(
                serde_ts_typing::Value::Null
            )),
        )
    }
}

fn derive_struct(input: ItemStruct) -> proc_macro2::TokenStream {
    let visitor = StructVisitor(&input);

    let struct_name = visitor.ident(); // input.ident;
    let where_clause = gen_where_clause(visitor.generics());
    let gparam = &visitor.0.generics.params;
    let gparam_pure = visitor.generics().get_pure_params();

    let ctxt = ContainerContext::from_attr(visitor.attrs());
    let ts_ctxt = ts_attr::ContainerContext::from_attr(visitor.attrs());

    let container_docs = visitor.attrs().get_docs();
    let (mut tyctxt, mut tydef) = gen_fields(&ctxt, visitor.fields());

    // 目前看来只有在结合了 tag 的时候有用
    let name = ctxt.rename(struct_name.to_string());
    // must be named fields
    if let Some(tag) = ctxt.tag() {
        tydef = quote!({
            let mut ty = #tydef;
            ty.struct_insert(#tag.into(), serde_ts_typing::TypeExpr::Value(serde_ts_typing::Value::String(#name.into())));
            ty
        });
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
                    c.register(id, #ty_name.into(), #tydef, #container_docs);
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
        impl<#gparam> serde_ts_typing::TsType for #struct_name<#gparam_pure> #where_clause {
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
    let visitor = EnumVisitor(&input);

    let enum_name = visitor.ident();
    let where_clause = gen_where_clause(visitor.generics());
    let gparam = &visitor.0.generics.params;
    let gparam_pure = visitor.generics().get_pure_params();

    let ctxt = ContainerContext::from_attr(visitor.attrs());
    let ts_ctxt = ts_attr::ContainerContext::from_attr(visitor.attrs());

    let enum_ty_name = ts_ctxt.name.clone().unwrap_or(enum_name.to_string());
    let mut tyctxts = Vec::new();
    let mut tydefs = Vec::new();

    for var in visitor.varients() {
        let var_ctxt = ctxt.provide_default(VariantContext::from_attr(var.attrs()));

        if var_ctxt.is_skip() {
            continue;
        }
        if var_ctxt.serialize_with || var_ctxt.with || var.0.discriminant.is_some() {
            unimplemented!()
        }
        if ctxt.untagged() {
            unimplemented!()
        }

        let var_name = var_ctxt.rename_variant(var.ident().to_string());
        let is_unit = matches!(var.0.fields, Fields::Unit);
        let (tyctxt, mut tydef) = gen_fields(&var_ctxt, var.fields());
        tyctxts.push(tyctxt);
        if let (Some(tag), Some(ctag)) = (ctxt.tag(), ctxt.content_tag()) {
            // adjacently tagged
            tydef = quote!(
                serde_ts_typing::TypeExpr::Struct([
                    (#tag.into(), serde_ts_typing::TypeExpr::Value(serde_ts_typing::Value::String(#var_name.into()))),
                    (#ctag.into(), #tydef),
                ].into_iter().collect())
            )
        } else if let Some(tag) = ctxt.tag() {
            // internally tagged
            // 如果是 unnamed fields，serde 不会编译报错，而会在运行时 panic
            if is_unit {
                tydef = quote!(
                    serde_ts_typing::TypeExpr::Struct([
                        (#tag.into(), serde_ts_typing::TypeExpr::Value(serde_ts_typing::Value::String(#var_name.into())))
                    ].into_iter().collect())
                )
            } else {
                // 对于 enum 里的 newtype(enum::unit) 可能会出问题
                tydef = quote!({
                    serde_ts_typing::TypeExpr::Intersection(
                        [
                            serde_ts_typing::TypeExpr::Struct( [
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
                tydef = quote!(serde_ts_typing::TypeExpr::Struct([(#var_name.into(), #tydef)].into_iter().collect()));
            }
        }

        if !ts_ctxt.variant_inline {
            let var_ty_name = enum_ty_name.clone() + &var.ident().to_string();
            let var_docs = var.attrs().get_docs();
            tyctxts.push(quote!({
                c.register_variant(#var_ty_name.into(), #tydef, #var_docs);
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
    let container_docs = visitor.attrs().get_docs();
    if !ts_ctxt.inline {
        tyctxt = {
            let mut head = quote!({
                let id = std::any::TypeId::of::<#enum_name>();
                if !c.contains(id) {
                    c.register(id, #enum_ty_name.into(), #tydef, #container_docs);
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
        impl<#gparam> serde_ts_typing::TsType for #enum_name<#gparam_pure> #where_clause {
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
