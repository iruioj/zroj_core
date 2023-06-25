use quote::quote;
use syn::{GenericParam, Item, ItemStruct};

fn derive_struct(input: ItemStruct) -> proc_macro2::TokenStream {
    let struct_name = input.ident;
    let generics = input.generics;
    let gparam = generics.params;

    let where_clause = {
        gparam
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
    };

    let type_def_stmt = match input.fields {
        syn::Fields::Named(fields) => {
            let mut tks = quote!(
                let mut r = String::from("{");
            );
            fields.named.iter().for_each(|f| {
                let name_str = f.ident.clone().unwrap().to_string();
                let ty = &f.ty;
                tks.extend(quote!(
                    r = r + #name_str + ": " + &<#ty as serde_ts_typing::TypeDef>::type_def() + ";";
                ))
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
    };

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

/// 为实现了 Serialize 的类型提供 typescript 类型生成
#[proc_macro_derive(SerdeJsonWithType)]
pub fn derive_serde_json_with_type(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: Item = syn::parse_macro_input!(item);
    if let Item::Struct(input) = input {
        derive_struct(input).into()
    } else {
        unimplemented!()
    }
}
