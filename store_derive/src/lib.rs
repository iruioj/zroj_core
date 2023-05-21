use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::Item;

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
        let ident = item.ident;
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

        for field in item.fields {
            if let Some(name) = &field.ident {
                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.meta.path().is_ident("meta"))
                {
                    // store in meta file
                    let ty = field.ty;
                    fields.extend(quote!( #name : #ty, ));
                    ret_fields.extend(quote!( #name : __meta__.#name, ));
                    into_meta_fields.extend(quote!( #name : self.#name.clone(), ));
                } else {
                    let name_str = name.to_string();
                    ret_fields.extend(quote!( #name : FsStore::open(path.join(#name_str))?, ));
                    save_block.extend(quote!( self.#name.save(path.join(#name_str))?; ));
                }
            } else {
                unimplemented!()
            }
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
                fn open(path: store::Handle) -> Result<Self, store::Error> {
                    use std::fs::File;
                    let __meta__: #meta_struct_ident =
                        path.join("__meta__").deserialize::<#meta_struct_ident>()?;

                    Ok(Self {
                        #ret_fields
                    })
                }
                fn save(&mut self, path: store::Handle) -> Result<(), store::Error> {
                    path.join("__meta__").serialize_new_file(& #meta_struct_ident {
                        #into_meta_fields
                    })?;
                    #save_block
                    Ok(())
                }
            }

            #meta_struct_stmt
        };

        output.extend(stmt);
    } else if let Item::Enum(_item) = input {
        unimplemented!()
    }

    output.into()
}
