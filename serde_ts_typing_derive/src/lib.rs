use quote::quote;
use syn::{
    AttrStyle, Expr, Fields, GenericParam, Generics, Item, ItemEnum, ItemStruct, Lit, Meta,
    MetaNameValue,
};

/// 对于带有泛型的类型，我们要求所有类型参数都实现 TypeDef
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
            quote!( #ident: serde_ts_typing::TypeDef, )
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

/// 对于 enum 或者 struct 的 fields 生成类型的方式是一样的
fn gen_fields_type(fields: Fields) -> proc_macro2::TokenStream {
    match fields {
        syn::Fields::Named(fields) => {
            let mut tks = quote!(
                let mut r = String::from("{");
            );
            fields.named.iter().for_each(|f| {
                let mut name_str = f.ident.clone().unwrap().to_string();
                let mut ty = &f.ty;
                f.attrs
                    .iter()
                    .filter_map(|a| {
                        if matches!(a.style, AttrStyle::Outer) {
                            if let Meta::List(list) = &a.meta {
                                if list.path.is_ident("serde") {
                                    return Some(&list.tokens);
                                }
                            }
                        }
                        None
                    })
                    .for_each(|tokens| {
                        let tokens = tokens.clone();
                        if let Ok(item) = syn::parse2::<MetaNameValue>(tokens) {
                            // rename = "..."
                            if item.path.is_ident("rename") {
                                if let Expr::Lit(lit) = item.value {
                                    if let Lit::Str(s) = lit.lit {
                                        name_str = s.value()
                                    }
                                }
                            }
                        }
                    });
                // Option<T>
                let mut is_option = false;
                if let syn::Type::Path(p) = ty {
                    if p.path.segments.len() == 1 {
                        let last = p.path.segments.last().unwrap();
                        if last.ident == "Option" {
                            if let syn::PathArguments::AngleBracketed(target) = &last.arguments {
                                assert!(target.args.len() == 1);
                                if let syn::GenericArgument::Type(target) = &target.args[0] {
                                    ty = target;
                                    is_option = true;
                                }
                            }
                        }
                    }
                }
                if is_option {
                    tks.extend(quote!(
                        r = r + #name_str + "?: " + &<#ty as serde_ts_typing::TypeDef>::type_def() + ";";
                    ))
                } else {
                    tks.extend(quote!(
                        r = r + #name_str + ": " + &<#ty as serde_ts_typing::TypeDef>::type_def() + ";";
                    ))
                }
            });
            tks.extend(quote!(
                r = r + "}";
                r
            ));
            tks
        }
        syn::Fields::Unnamed(fields) => {
            let mut tuple_type = proc_macro2::TokenStream::new();
            // newtype struct
            if fields.unnamed.len() == 1 {
                let ty = &fields.unnamed.first().unwrap().ty;
                quote!(<#ty as serde_ts_typing::TypeDef>::type_def())
            }
            // tuple struct
            else {
                fields
                    .unnamed
                    .iter()
                    .map(|f| &f.ty)
                    .for_each(|t| tuple_type.extend(quote!(#t, )));
                quote!(<( #tuple_type ) as serde_ts_typing::TypeDef>::type_def())
            }
        }
        syn::Fields::Unit => panic!("unit struct dosen't have type"),
    }
}

fn derive_struct(input: ItemStruct) -> proc_macro2::TokenStream {
    let struct_name = input.ident;
    let generics = input.generics;
    let where_clause = gen_where_clause(generics.clone());
    let gparam = generics.params;

    let type_def_stmt = gen_fields_type(input.fields);

    quote! {
        impl<#gparam> serde_ts_typing::TypeDef for #struct_name<#gparam> #where_clause {
            fn type_def() -> String {
                #type_def_stmt
            }
        }
        impl<#gparam> SerdeJsonWithType for #struct_name<#gparam> #where_clause {
        }
    }
}

fn derive_enum(input: ItemEnum) -> proc_macro2::TokenStream {
    let enum_name = input.ident;
    let generics = input.generics;
    let where_clause = gen_where_clause(generics.clone());
    let gparam = generics.params;

    let mut type_def_stmt = quote!(
        let mut rs = String::new();
    );
    let mut is_first = true;
    input
        .variants
        .into_iter()
        .map(|var| {
            let varient_name = var.ident.to_string();
            if matches!(var.fields, Fields::Unit) {
                let varient_name = String::from("\"") + &varient_name + "\"";
                return if is_first {
                    is_first = false;
                    quote!(rs = rs + #varient_name;)
                } else {
                    quote!(rs = rs + " | " + #varient_name;)
                };
            }

            let fields_ty = gen_fields_type(var.fields);

            if is_first {
                is_first = false;
                quote!(
                    rs = rs + "{" + #varient_name + ": " + &{
                        #fields_ty
                    } + "}";
                )
            } else {
                quote!(
                    rs = rs + " | " + "{" + #varient_name + ": " + &{
                        #fields_ty
                    } + "}";
                )
            }
        })
        .for_each(|tks| type_def_stmt.extend(tks));
    type_def_stmt.extend(quote!(rs));

    quote! {
        impl<#gparam> serde_ts_typing::TypeDef for #enum_name<#gparam> #where_clause {
            fn type_def() -> String {
                #type_def_stmt
            }
        }
        impl<#gparam> SerdeJsonWithType for #enum_name<#gparam> #where_clause {
        }
    }
}

/// 为实现了 Serialize 的类型提供 typescript 类型生成
#[proc_macro_derive(SerdeJsonWithType)]
pub fn derive_serde_json_with_type(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: Item = syn::parse_macro_input!(item);
    if let Item::Struct(input) = input {
        derive_struct(input).into()
    } else if let Item::Enum(input) = input {
        derive_enum(input).into()
    } else {
        unimplemented!()
    }
}
