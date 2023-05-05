use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Expr, ItemFn, Meta, Stmt, Token};

#[derive(Debug)]
struct ScopeServiceAttr {
    path: Option<Expr>,
}

impl Parse for ScopeServiceAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut res = Self { path: None };
        // parse meta list
        let metas = input.parse_terminated(Meta::parse, Token![,])?;
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
///   ```rust
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
pub fn scope_service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let it = item.clone();
    let itemfn: ItemFn = parse_macro_input!(it);

    let name = itemfn.sig.ident;
    let params = itemfn.sig.inputs;
    let vis = itemfn.vis;
    let body = itemfn.block;

    let mut bodys = Vec::new();
    body.stmts.into_iter().for_each(|stmt| {
        if let Stmt::Expr(Expr::Call(expr), Some(_)) = &stmt {
            if let Expr::Path(e) = &*expr.func {
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
                    bodys.push(quote!( let scope = scope.#e ( #args ); ));
                    return;
                }
            }
        }
        bodys.push(quote!( #stmt ));
    });
    let bodys = bodys.into_iter().reduce(|mut acc, e| {
        acc.extend(e);
        acc
    });

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
    };
    ret.into()
}
