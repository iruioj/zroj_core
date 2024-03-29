use server::web::services::DOCS;
use std::collections::BTreeMap;

fn indent(content: &str, indent: usize) -> String {
    let indent_prefix = " ".repeat(indent);
    content
        .split('\n')
        .map(|ln| indent_prefix.clone() + ln)
        .reduce(|a, b| a + "\n" + &b)
        .unwrap_or_default()
}
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
        docs: String,
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
                    acc.0 + "\n" + &cur.0,
                    acc.1.into_iter().chain(cur.1).collect(),
                )
            })
            .unwrap();

        format!(
            r#"
export function useAPI () {{
    return {{
{}
    }};
}}
{}
"#,
            indent(&code, 8),
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
                let EntryNode::Path { slug, children } = c else {
                    return c;
                };
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
                docs,
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
                            r#"/**
{docs} */
{method}: {{ 
    use: (payload: {path_ty} | Ref<{path_ty}>) => callAPI({method:?}, {path:?}, payload) as Promise<ExtAsyncData<{ret_ty}>>,
    fetch: (payload: {path_ty} | Ref<{path_ty}>) => fetchAPI({method:?}, {path:?}, payload) as Promise<{}>,
    key: {:?},
}},"#,
                            o_ret_ty.unwrap_or("void".into()),
                            path.to_owned() + ":" + method,
                        ),
                        ty,
                    )
                } else {
                    (
                        format!(
                            r#"{method}: {{
    use: () => callAPI({method:?}, {path:?}) as Promise<ExtAsyncData<{ret_ty}>>,
    fetch: () => fetchAPI({method:?}, {path:?}) as Promise<{}>,
    key: {:?},
}},"#,
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
                            acc.0 + "\n" + &cur.0,
                            acc.1.into_iter().chain(cur.1).collect(),
                        )
                    })
                    .unwrap();
                (format!("{slug}: {{\n{}\n}},", indent(&inner.0, 4)), inner.1)
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
                docs: api.description,
                returns: api.res_type.map(|ty| ty.to_string()),
            },
        );
    }
    assert!(children.len() == 1);
    children.remove(0)
}

fn main() {
    let entry = EntryRoot(DOCS.iter().map(|o| gen_entry(o.0.clone())).collect());

    let code = String::from(
        r#"// generated by server/src/bin/gen_docs.rs
// DO NOT EDIT.

import { callAPI, fetchAPI, type ExtAsyncData } from "./inner/fetch";

"#,
    ) + &DOCS
        .iter()
        .map(|o| o.1.clone())
        .reduce(|a, b| a + b)
        .unwrap_or_default()
        .render_code(4)
        + &entry.gen_code();
    println!("{code}");
}
