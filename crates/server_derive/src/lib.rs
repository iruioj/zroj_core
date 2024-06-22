#![doc = include_str!("../README.md")]

use convert_case::Casing;
use quote::{format_ident, quote, ToTokens};
use structural_macro_utils::AttrListVisitor;
use syn::{parse::Parse, parse_macro_input, Expr, FnArg, ItemFn, Meta, ReturnType, Stmt, Token};

struct AttrList(Vec<Meta>);

impl Parse for AttrList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let metas = input.parse_terminated(Meta::parse, Token![,])?;
        Ok(AttrList(metas.into_iter().collect()))
    }
}

struct ScopeServiceAttr {
    path: Option<Expr>,
}

impl Parse for ScopeServiceAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut res = Self { path: None };
        let metas = AttrList::parse(input)?.0;
        for meta in metas {
            if let Meta::NameValue(meta) = meta {
                if meta.path.is_ident("path") {
                    let value = meta.value;
                    res.path = Some(value);
                }
            }
        }
        Ok(res)
    }
}

/// Custom REST service macro based on actix_web
///
/// - use `path = "..."` the specify the path of this service
/// - 函数内部自动新建一个 `web::Scope` 类型的变量 `scope`
/// - 函数内调用 `service`，`app_data`，`wrap`，`wrap_fn`，`route`，`default_service`，`guard`，`configure`
///   函数（末尾需要带分号）将会自动转换为 scope 上的方法，使用 `r#` 可以取消转换
/// - the return value would be enforced to
///   ```no-run rust
///   actix_web::Scope<
///       impl actix_web::dev::ServiceFactory<
///           actix_web::dev::ServiceRequest,
///           Config = (),
///           Response = actix_web::dev::ServiceResponse,
///           Error = actix_web::Error,
///           InitError = (),
///       >,
///   >
///   ```
/// - 生成一个 document metadata generator
#[proc_macro_attribute]
pub fn scope_service(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let itemfn: ItemFn = parse_macro_input!(item);

    let name = itemfn.sig.ident;
    let doc_name = format_ident!("{}_doc", name);
    let params = itemfn.sig.inputs;
    let vis = itemfn.vis;
    let body = itemfn.block;

    let mut bodys = proc_macro2::TokenStream::new();
    let mut doc_stmt = quote!(let mut docs = Vec::new(););
    body.stmts.into_iter().for_each(|stmt| {
        if let Stmt::Expr(Expr::Call(expr), Some(_)) = &stmt {
            if let Expr::Path(e) = &*expr.func {
                if e.path.is_ident("service") {
                    let name = expr.args.first().unwrap();
                    if let Expr::Path(p) = name {
                        let name = p.path.to_token_stream().to_string();
                        let api_doc_name = format_ident!("{}_doc", name);
                        doc_stmt.extend(quote!({
                            let (doc, c) = #api_doc_name();
                            docs.push(doc);
                            ctxt = ctxt + c;
                        }));
                    } else {
                        panic!("{:?}", name)
                    }
                }
                if e.path.is_ident("app_data")
                    || e.path.is_ident("configure")
                    || e.path.is_ident("default_service")
                    || e.path.is_ident("guard")
                    || e.path.is_ident("route")
                    || e.path.is_ident("service")
                    || e.path.is_ident("wrap")
                    || e.path.is_ident("wrap_fn")
                {
                    let args = &expr.args;
                    bodys.extend(quote!( let scope = scope.#e ( #args ); ));
                    return;
                }
            }
        }
        bodys.extend(quote!( #stmt ));
    });
    doc_stmt.extend(quote!(docs));

    // keep other attributes
    let attrs = itemfn
        .attrs
        .into_iter()
        .map(|a| quote!( #a ))
        .reduce(|mut acc, e| {
            acc.extend(e);
            acc
        });

    let ScopeServiceAttr { path } = parse_macro_input!(attr);

    let ret = quote! {
        /// documentation metadata
        pub fn #doc_name() -> (crate::ServiceDoc, serde_ts_typing::Context) {
            let path = String::from(#path);
            let mut ctxt = serde_ts_typing::Context::default();
            let apis = {
                #doc_stmt
            };
            (crate::ServiceDoc { path, apis }, ctxt)
        }

        #attrs
        #vis fn #name (#params) -> actix_web::Scope<
            impl actix_web::dev::ServiceFactory<
                actix_web::dev::ServiceRequest,
                Config = (),
                Response = actix_web::dev::ServiceResponse,
                Error = actix_web::Error,
                InitError = (),
            >,
        > {
            let scope = actix_web::web::scope(#path);

            #bodys

            scope
        }
    };
    ret.into()
}

struct ApiConfig {
    method: String,
    path: String,
}
impl Parse for ApiConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut method = String::new();
        let mut path = String::new();

        let list = AttrList::parse(input)?;
        list.0.into_iter().for_each(|m| match m {
            Meta::Path(_) => todo!(),
            Meta::List(_) => todo!(),
            Meta::NameValue(meta) => {
                if meta.path.is_ident("method") {
                    method = meta.value.into_token_stream().to_string();
                } else if meta.path.is_ident("path") {
                    path = meta.value.into_token_stream().to_string();
                    path = path[1..path.len() - 1].to_string();
                }
            }
        });

        Ok(Self { method, path })
    }
}

/// 解析形如 `Marker<InnerType>` 的类型
fn parse_marker_type(marker: impl AsRef<str>, ty: &syn::Type) -> Option<syn::TypePath> {
    if let syn::Type::Path(ty) = ty {
        // 粗暴，只看最后一个是不是和 marker 一样
        let last = ty.path.segments.last().unwrap();
        if last.ident == marker.as_ref() {
            if let syn::PathArguments::AngleBracketed(g) = &last.arguments {
                let target = g.args.first().unwrap();
                if let syn::GenericArgument::Type(syn::Type::Path(target)) = target {
                    return Some(target.clone());
                }
            }
        }
    }
    None
}

/// An enhancement of the macros in `actix_web`,
/// Use this macro to define a request handler and a document metadata generator function.
///
/// - use `method = xxx` to declare HTTP method
/// - use `path = "xxx"` to declare API path
///
#[proc_macro_attribute]
pub fn api(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config: ApiConfig = parse_macro_input!(attr);

    let func: ItemFn = parse_macro_input!(item);
    let method_ident = format_ident!("{}", config.method);
    let method_str = method_ident.to_string();
    let path = config.path;

    let mut ctxt_stmt = quote!(let mut c = serde_ts_typing::Context::default(););
    let mut body_type_stmt = quote!(let body_type = None;);
    let mut query_type_stmt = quote!(let query_type = None;);
    let mut is_form = false;

    // parse marker types of parameters
    func.sig
        .inputs
        .clone()
        .into_iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) => Some(t),
        })
        .for_each(|arg| {
            if let Some(v) = parse_marker_type("JsonBody", &arg.ty) {
                body_type_stmt =
                    quote!(let body_type = Some(<#v as serde_ts_typing::TsType>::type_def()););
                ctxt_stmt.extend(quote!(
                    <#v as serde_ts_typing::TsType>::register_context(&mut c);
                ))
            } else if let Some(v) = parse_marker_type("QueryParam", &arg.ty) {
                query_type_stmt =
                    quote!(let query_type = Some(<#v as serde_ts_typing::TsType>::type_def()););
                ctxt_stmt.extend(quote!(
                    <#v as serde_ts_typing::TsType>::register_context(&mut c);
                ))
            } else if parse_marker_type("FormData", &arg.ty).is_some() {
                is_form = true;
            }
        });

    let mut res_type_stmt = quote!(let res_type = None;);
    if let ReturnType::Type(_, ty) = &func.sig.output {
        if let Some(v) = parse_marker_type("JsonResult", ty) {
            res_type_stmt =
                quote!(let res_type = Some(<#v as serde_ts_typing::TsType>::type_def()););
            ctxt_stmt.extend(quote!(
                <#v as serde_ts_typing::TsType>::register_context(&mut c);
            ))
        } else if parse_marker_type("AnyResult", ty).is_some() {
            res_type_stmt = quote!(let res_type = Some(serde_ts_typing::TypeExpr::Any););
        }
    }

    let attr_visitor = AttrListVisitor(&func.attrs);
    let descrip_stmt = attr_visitor.get_docs();
    let doc_name = format_ident!("{}_doc", func.sig.ident);

    let ret = quote! {
        fn #doc_name() -> (crate::ApiDocMeta, serde_ts_typing::Context) {
            let path = String::from(#path);
            let method = String::from(#method_str);
            #body_type_stmt
            #res_type_stmt
            #query_type_stmt
            (crate::ApiDocMeta {
                path,
                method,
                query_type,
                body_type,
                is_form: #is_form,
                res_type,
                description: #descrip_stmt
            }, {
                #ctxt_stmt
                c
            })
        }

        #[actix_web:: #method_ident(#path)]
        #func
    };

    ret.into()
}

#[proc_macro_attribute]
pub fn perm_guard(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut imp: syn::ItemImpl = parse_macro_input!(item);
    let impty = match &*imp.self_ty {
        syn::Type::Path(p) => p,
        _ => panic!("invalid perm_guard implementation"),
    };
    let impname = &impty.path.segments.last().unwrap().ident;
    let mut extra_statements = quote!();
    let mut extra_imp_items = quote!();

    imp.items.iter_mut().for_each(|item| {
        let func = match item {
            syn::ImplItem::Fn(func) => func,
            _ => return,
        };
        // find private function
        let syn::Visibility::Inherited = func.vis else {
            return;
        };
        let mut id_ty = None;
        let mut id_fname = None;
        let fnname = &func.sig.ident;

        let perm_id_varname = format_ident!(
            "{impname}{}",
            fnname.to_string().to_case(convert_case::Case::Pascal)
        );

        let rs_fnname = format_ident!("rs_{fnname}");

        let mut rs_fields = quote!();
        let mut call_self_args = quote!();
        let mut call_args = quote!();

        func.sig.inputs.iter_mut().for_each(|arg| {
            let arg = match arg {
                FnArg::Receiver(_) => return,
                FnArg::Typed(r) => r,
            };
            let mut is_id = false;
            for i in (0..arg.attrs.len()).rev() {
                if arg.attrs[i].meta.path().is_ident("id") {
                    arg.attrs.remove(i);
                    is_id = true;
                    break;
                }
            }
            if is_id {
                id_ty = Some(arg.ty.clone());
                id_fname = Some(arg.pat.clone());
            }

            let fname = &arg.pat;
            let fty = &arg.ty;
            rs_fields.extend(quote!(#fname : #fty, ));
            call_self_args.extend(quote!(self.#fname, ));
            call_args.extend(quote!(#fname, ));
        });
        let retitem = match &func.sig.output {
            ReturnType::Default => return,
            ReturnType::Type(_, ty) => match &**ty {
                syn::Type::Path(p) => {
                    let args = &p.path.segments.last().unwrap().arguments;
                    match args {
                        syn::PathArguments::AngleBracketed(p) => match p.args.first() {
                            Some(syn::GenericArgument::Type(ty)) => ty,
                            _ => return,
                        },
                        _ => return,
                    }
                }
                _ => return,
            },
        };
        if id_fname.is_none() {
            panic!("no id attributed")
        }
        let rs_struct = quote!(
            pub struct #perm_id_varname<'a> {
                #rs_fields
                db: &'a #impname,
            }

            impl<'a> Resource for #perm_id_varname<'a> {
                type Item = #retitem;

                fn perm_id(&self) -> PermID {
                    PermID::#perm_id_varname(self.#id_fname)
                }

                fn load(&self) -> Result<Self::Item, DataError> {
                    self.db.#fnname(#call_self_args)
                }
            }
        );
        extra_statements.extend(rs_struct);
        extra_imp_items.extend(quote!(
            pub fn #rs_fnname(&self, #rs_fields) -> ResourceHandle<#perm_id_varname> {
                ResourceHandle::new(#perm_id_varname { #call_args db: self })
            }
        ));
    });

    let ret = quote! {
        #imp
        impl #impname {
            #extra_imp_items
        }
        #extra_statements
    };

    ret.into()
}
