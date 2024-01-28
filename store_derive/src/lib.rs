use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Attribute, Item};

fn has_meta(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.meta.path().is_ident("meta"))
}

/// this character is almost impossible to appear as the name of field in Rust structs.
/// Thus we use it as the metadata file's basename.
const SELF_SEG: &str = "@";

/// 可将文件夹下文件保存的信息自动初始化到结构体中
///
/// `meta` 属性表示将此字段保存到一个统一的元数据文件中，此字段类型需要实现 Serialize 和 Deserialize
///
/// 没有 `meta` 属性的字段类型需要实现 FsStore
///
/// 目前不支持 enum 和匿名字段的结构体（即 struct(A, B)），需要手动实现
///
/// 需要 serde/derive
#[proc_macro_derive(FsStore, attributes(meta))]
pub fn derive_fs_store(item: TokenStream) -> TokenStream {
    let input: Item = syn::parse_macro_input!(item);
    let mut output = proc_macro2::TokenStream::new();

    if let Item::Struct(item) = input {
        let ident: Ident = item.ident;
        let mut fields = proc_macro2::TokenStream::new();
        let mut ret_fields = proc_macro2::TokenStream::new();
        let mut into_meta_fields = proc_macro2::TokenStream::new();
        let mut save_block = proc_macro2::TokenStream::new();

        let syn::Generics {
            lt_token: _,
            params,
            gt_token: _,
            where_clause,
        } = item.generics;

        match item.fields {
            syn::Fields::Named(itemfields) => {
                for field in itemfields.named {
                    let name = field.ident.unwrap();
                    if has_meta(&field.attrs) {
                        // store in meta file
                        let ty = field.ty;
                        fields.extend(quote!( #name : #ty, ));
                        ret_fields.extend(quote!( #name : __meta__.#name, ));
                        into_meta_fields.extend(quote!( #name : self.#name.clone(), ));
                    } else {
                        let name_str = name.to_string();
                        ret_fields.extend(quote!( #name : FsStore::open(&path.join(#name_str))?, ));
                        save_block.extend(quote!( self.#name.save(&path.join(#name_str))?; ));
                    }
                }
            }
            syn::Fields::Unnamed(_) => todo!(),
            syn::Fields::Unit => todo!(),
        }

        let meta_struct_ident =
            Ident::new((ident.to_string() + "__Meta").as_str(), Span::call_site());
        let meta_struct_stmt = quote! {
            #[derive(store::SerdeSerialize, store::SerdeDeserialize)]
            #[allow(non_camel_case_types)]
            struct #meta_struct_ident {
                #fields
            };
        };
        let stmt = quote! {
            #[automatically_derived]
            impl<#params> FsStore for #ident<#params> #where_clause {
                fn open(path: &store::Handle) -> Result<Self, store::Error> {
                    use std::fs::File;
                    let __meta__: #meta_struct_ident =
                        path.join(#SELF_SEG).deserialize::<#meta_struct_ident>()?;

                    Ok(Self {
                        #ret_fields
                    })
                }
                fn save(&mut self, path: &store::Handle) -> Result<(), store::Error> {
                    path.join(#SELF_SEG).serialize_new_file(& #meta_struct_ident {
                        #into_meta_fields
                    })?;
                    #save_block
                    Ok(())
                }
            }

            #meta_struct_stmt
        };

        output.extend(stmt);
    }
    // enum
    else if let Item::Enum(item) = input {
        let ident = item.ident;

        let syn::Generics {
            lt_token: _,
            params,
            gt_token: _,
            where_clause,
        } = item.generics;

        let mut save_branches = proc_macro2::TokenStream::new();
        let mut meta_branches = proc_macro2::TokenStream::new();
        let mut ret_branches = proc_macro2::TokenStream::new();

        let meta_enum_ident =
            Ident::new((ident.to_string() + "__Meta").as_str(), Span::call_site());

        for variant in item.variants {
            let variant_meta = has_meta(&variant.attrs);
            let varname = variant.ident;

            assert!(variant.discriminant.is_none());

            match variant.fields {
                syn::Fields::Named(varfields) => {
                    let mut save_meta_fields = proc_macro2::TokenStream::new();
                    let mut fields = proc_macro2::TokenStream::new();
                    let mut ret_fields = proc_macro2::TokenStream::new();
                    let mut save_block = proc_macro2::TokenStream::new();

                    let mut meta_fieldnames: Punctuated<Ident, Comma> = Punctuated::new();
                    let mut fieldnames: Punctuated<Ident, Comma> = Punctuated::new();
                    for field in varfields.named {
                        let name = field.ident.unwrap();
                        fieldnames.push(name.clone());
                        if variant_meta || has_meta(&field.attrs) {
                            // store in meta file
                            let ty = field.ty;
                            fields.extend(quote!( #name : #ty, ));
                            ret_fields.extend(quote!( #name : #name, ));
                            save_meta_fields.extend(quote!( #name : #name.clone(), ));
                            meta_fieldnames.push(name);
                        } else {
                            let name_str = name.to_string();
                            ret_fields
                                .extend(quote!( #name : FsStore::open(&path.join(#name_str))?, ));

                            save_block.extend(quote!( #name.save(&path.join(#name_str))?; ));
                        }
                    }
                    meta_branches.extend(quote!(#varname{#fields},));
                    save_branches.extend(quote!(
                        #ident::#varname{#fieldnames} => {
                            #save_block

                            #meta_enum_ident::#varname{#save_meta_fields}
                        },
                    ));
                    ret_branches.extend(quote!(
                        #meta_enum_ident::#varname{#meta_fieldnames} => #ident::#varname{#ret_fields},
                    ));
                }
                syn::Fields::Unnamed(_) => todo!(),
                syn::Fields::Unit => {
                    meta_branches.extend(quote!(#varname,));
                    save_branches.extend(quote!( #ident::#varname => #meta_enum_ident::#varname, ));
                    ret_branches.extend(quote!( #meta_enum_ident::#varname => #ident::#varname,));
                }
            }
        }

        let meta_enum_stmt = quote! {
            #[derive(store::SerdeSerialize, store::SerdeDeserialize)]
            #[allow(non_camel_case_types)]
            enum #meta_enum_ident {
                #meta_branches
            };
        };
        let stmt = quote! {
            #[automatically_derived]
            impl<#params> FsStore for #ident<#params> #where_clause {
                fn open(path: &store::Handle) -> Result<Self, store::Error> {
                    use std::fs::File;
                    let __meta__: #meta_enum_ident =
                        path.join(#SELF_SEG).deserialize::<#meta_enum_ident>()?;

                    Ok(match __meta__ {
                        #ret_branches
                    })
                }
                fn save(&mut self, path: &store::Handle) -> Result<(), store::Error> {
                    path.join(#SELF_SEG).serialize_new_file(& match self {
                        #save_branches
                    })?;
                    Ok(())
                }
            }

            #meta_enum_stmt
        };
        output.extend(stmt);
    }

    output.into()
}
