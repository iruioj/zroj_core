use server::app;
use std::collections::BTreeMap;

fn to_typename(path: String) -> String {
    path.split('/').fold(String::new(), |acc, s| {
        acc + &s.split('_').fold(String::new(), |acc, s| {
            let mut c = s.chars();
            acc + &match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
    })
}

#[derive(Clone, Debug)]
enum EntryNode {
    Endpoint {
        method: String,
        payload: Option<String>,
        returns: Option<String>,
    },
    Path {
        slug: String,
        children: Vec<EntryNode>,
    },
}

#[derive(Clone, Debug)]
struct EntryRoot(Vec<EntryNode>);

impl EntryRoot {
    fn gen_code(&self) -> String {
        let (code, types) = self
            .0
            .iter()
            .map(|s| s.gen_code(&String::new()))
            .reduce(|acc, cur| {
                (
                    acc.0 + &cur.0,
                    acc.1.into_iter().chain(cur.1.into_iter()).collect(),
                )
            })
            .unwrap();

        format!(
            "export function useAPI () {{ return {{ {code} }}; }}\n{}",
            types
                .into_iter()
                .map(|(k, v)| format!("export type {k} = {v};\n"))
                .fold(String::new(), |acc, cur| acc + &cur)
        )
    }
}

impl EntryNode {
    fn replace_children(
        mut children: Vec<EntryNode>,
        slugs: &[String],
        endpoint: EntryNode,
    ) -> Vec<EntryNode> {
        if slugs.is_empty() {
            children.push(endpoint);
            return children;
        }
        let mut flag = false;
        let mut children: Vec<EntryNode> = children
            .into_iter()
            .map(|c| {
                let EntryNode::Path { slug, children } = c else { return c };
                if slug == slugs[0] {
                    flag = true;
                    EntryNode::Path {
                        slug,
                        children: EntryNode::replace_children(
                            children,
                            &slugs[1..],
                            endpoint.clone(),
                        ),
                    }
                } else {
                    EntryNode::Path { slug, children }
                }
            })
            .collect();
        if !flag {
            children.push(EntryNode::Path {
                slug: slugs[0].clone(),
                children: EntryNode::replace_children(Vec::new(), &slugs[1..], endpoint),
            });
        }
        children
    }
    // 返回 api code 和 type context
    fn gen_code(&self, path: &String) -> (String, BTreeMap<String, String>) {
        match self {
            EntryNode::Endpoint {
                method,
                payload,
                returns,
            } => {
                let mut ty = BTreeMap::new();
                let o_ret_ty = returns.clone().map(|ret| {
                    let ret_ty = to_typename(path.clone() + "/" + method + "/return");
                    ty.insert(ret_ty.clone(), ret);
                    ret_ty
                });
                let ret_ty = &o_ret_ty
                    .clone()
                    .map(|s| s + " | null")
                    .unwrap_or("void".into());
                if let Some(payload) = payload {
                    let path_ty = to_typename(path.clone() + "/" + method + "/payload");
                    ty.insert(path_ty.clone(), payload.clone());
                    (
                        format!(
                            r#"{method}: {{ 
    use: (payload: {path_ty}) => callAPI({method:?}, {path:?}, payload) as Promise<AsyncData<{ret_ty}, FetchError>>,
    fetch: (payload: {path_ty}) => fetchAPI({method:?}, {path:?}, payload) as Promise<{}>,
    key: {:?},
}},
"#,
                            o_ret_ty.unwrap_or("void".into()),
                            path.to_owned() + ":" + method,
                        ),
                        ty,
                    )
                } else {
                    (
                        format!(
                            r#"{method}: {{
    use: () => callAPI({method:?}, {path:?}) as Promise<AsyncData<{ret_ty}, FetchError>>,
    fetch: () => fetchAPI({method:?}, {path:?}) as Promise<{}>,
    key: {:?},
}},
"#,
                            o_ret_ty.unwrap_or("void".into()),
                            path.to_owned() + ":" + method,
                        ),
                        ty,
                    )
                }
            }
            EntryNode::Path { slug, children } => {
                let inner = children
                    .iter()
                    .map(|c| c.gen_code(&(path.clone() + "/" + slug)))
                    .reduce(|acc, cur| {
                        (
                            acc.0 + &cur.0,
                            acc.1.into_iter().chain(cur.1.into_iter()).collect(),
                        )
                    })
                    .unwrap();
                (format!("{slug}: {{ {} }},\n", inner.0), inner.1)
            }
        }
    }
}

/// 添加 overload function declaration
fn gen_entry(service: server::ServiceDoc) -> EntryNode {
    let mut children = Vec::new();
    for api in service.apis {
        // some invalid case
        if api.query_type.is_some() && (api.body_type.is_some() || api.is_form) {
            panic!("query conflict with body/form payload")
        }
        if api.query_type.is_some() && api.method != "get" {
            panic!("query should not be used for non-get api")
        }
        if api.method == "get" && (api.body_type.is_some() || api.is_form) {
            panic!("body/form should not be used for get api")
        }

        let path = service.path.clone() + &api.path;
        let slugs: Vec<String> = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        children = EntryNode::replace_children(
            children,
            &slugs,
            EntryNode::Endpoint {
                method: api.method.clone(),
                payload: if api.is_form {
                    Some("FormData".into())
                } else {
                    api.body_type.or(api.query_type).map(|ty| ty.to_string())
                },
                returns: api.res_type.map(|ty| ty.to_string()),
            },
        );
    }
    assert!(children.len() == 1);
    children.remove(0)
}

fn gen_nuxt_basic() -> String {
    r#"function callAPI(method: string, path: string, args?: any): Promise<any> {
    if (process.client) {
        console.log('client call api', method, path, args)
    }
    path = useRuntimeConfig().public.apiBase + path;

    const options = {
        server: false, // 这只会降低首次加载的体验
        key: method + ":" + path,
        method: method as any,
        credentials: 'include' as any,
        headers: useRequestHeaders()
    };
    if (args === undefined) {
        return useFetch(path, options);
    } else if (method === 'get') {
        return useFetch(path, { ...options, query: args });
    } else if (args instanceof FormData) {
        return useFetch(path, { ...options, body: args});
    } else {
        return useFetch(path, { ...options, body: args });
    }
}
function fetchAPI(method: string, path: string, args?: any): Promise<any> {
    if (process.client) {
        console.log('client call api', method, path, args)
    }
    path = useRuntimeConfig().public.apiBase + path;

    const options = {
        method: method as any,
        credentials: 'include' as any,
        headers: useRequestHeaders()
    };
    if (args === undefined) {
        return $fetch(path, options);
    } else if (method === 'get') {
        return $fetch(path, { ...options, query: args });
    } else if (args instanceof FormData) {
        return $fetch(path, { ...options, body: args});
    } else {
        return $fetch(path, { ...options, body: args });
    }
}
"#
    .into()
}

fn main() {
    let auth = app::auth::service_doc();
    let user = app::user::service_doc();
    let problem = app::problem::service_doc();
    let oneoff = app::one_off::service_doc();

    let entry = EntryRoot(vec![
        gen_entry(auth.0),
        gen_entry(user.0),
        gen_entry(problem.0),
        gen_entry(oneoff.0),
    ]);

    let code = String::from(
        r#"// generated by server/src/bin/gen_docs.rs
// DO NOT EDIT.

import type { AsyncData } from "nuxt/app";
import type { FetchError } from "ofetch";

"#,
    ) + &gen_nuxt_basic()
        + &(auth.1 + user.1 + problem.1 + oneoff.1).render_code()
        + &entry.gen_code();
    println!("{code}");
}
