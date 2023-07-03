use quote::{format_ident, quote, ToTokens};
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

/// 基于 actix_web 定义的 zroj 定制 service 宏
///
/// - 使用 `path = "..."` 设置路径
/// - 函数内部自动新建一个 `web::Scope` 类型的变量 `scope`
/// - 函数内调用 `service`，`app_data`，`wrap`，`wrap_fn`，`route`，`default_service`，`guard`，`configure`
///   函数（末尾需要带分号）将会自动转换为 scope 上的方法，使用 `r#` 可以取消转换
/// - 将返回值强制设置为
///   ```ignore
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
                        doc_stmt.extend(quote!(docs.push(#api_doc_name());));
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

        /// documentation metadata
        pub fn #doc_name() -> crate::ServiceDoc {
            let path = String::from(#path);
            let apis = {
                #doc_stmt
            };
            crate::ServiceDoc { path, apis }
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
fn parse_marker_type(marker: impl AsRef<str>, ty: syn::Type) -> Option<syn::TypePath> {
    if let syn::Type::Path(ty) = ty {
        // 粗暴，只看最后一个是不是和 marker 一样
        let last = ty.path.segments.last().unwrap();
        if last.ident.to_string() == marker.as_ref() {
            if let syn::PathArguments::AngleBracketed(g) = &last.arguments {
                let target = g.args.first().unwrap();
                if let syn::GenericArgument::Type(target) = target {
                    if let syn::Type::Path(target) = target {
                        return Some(target.clone());
                    }
                }
            }
        }
    }
    None
}

/// actix_web macro 的整合
///
/// - 使用 `method = xxx` 来声明 REST API 的 http 方法
/// - 使用 `path = "xxx"` 声明 API 路径
///
/// 并自动生成文档数据
#[proc_macro_attribute]
pub fn api(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config: ApiConfig = parse_macro_input!(attr);

    let func: ItemFn = parse_macro_input!(item);
    let doc_name = format_ident!("{}_doc", func.sig.ident);
    let method_ident = format_ident!("{}", config.method);
    let method_str = method_ident.to_string();
    let path = config.path;

    let mut body_type_stmt = quote!(let body_type = None;);
    let mut query_type_stmt = quote!(let query_type = None;);

    func.sig
        .inputs
        .clone()
        .into_iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) => Some(t),
        })
        .for_each(|arg| {
            if let Some(v) = parse_marker_type("JsonBody", *arg.ty.clone()) {
                body_type_stmt =
                    quote!(let body_type = Some(<#v as serde_ts_typing::TypeDef>::type_def()););
            } else if let Some(v) = parse_marker_type("QueryParam", *arg.ty) {
                query_type_stmt =
                    quote!(let query_type = Some(<#v as serde_ts_typing::TypeDef>::type_def()););
            }
        });

    let mut res_type_stmt = quote!(let res_type = None;);
    if let ReturnType::Type(_, ty) = &func.sig.output {
        if let Some(v) = parse_marker_type("JsonResult", *ty.clone()) {
            res_type_stmt =
                quote!(let res_type = Some(<#v as serde_ts_typing::TypeDef>::type_def()););
        }
    }

    let mut descrip_stmt = quote!(let mut description = String::new(););
    func.attrs
        .clone()
        .into_iter()
        .filter_map(|f| match f.meta {
            Meta::NameValue(v) => Some(v),
            _ => None,
        })
        .filter_map(|v| {
            if v.path.is_ident("doc") {
                Some(v.value)
            } else {
                None
            }
        })
        .for_each(|e| descrip_stmt.extend(quote!( description += #e; )));

    let ret = quote! {
        #[actix_web:: #method_ident(#path)]
        #func

        fn #doc_name() -> crate::ApiDocMeta {
            let path = String::from(#path);
            let method = String::from(#method_str);
            #body_type_stmt
            #res_type_stmt
            #descrip_stmt
            #query_type_stmt
            crate::ApiDocMeta {
                path,
                method,
                query_type,
                body_type,
                res_type,
                description
            }
        }
    };

    ret.into()
}
