use server::app;

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
        let code = self
            .0
            .iter()
            .map(|s| s.gen_code(String::new()))
            .reduce(|acc, cur| acc + &cur)
            .unwrap();

        format!("export function useAPI () {{ return {{ {code} }}; }}")
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
    fn gen_code(&self, path: String) -> String {
        match self {
            EntryNode::Endpoint {
                method,
                payload,
                returns,
            } => {
                if let Some(payload) = payload {
                    format!(
                        "{}: (payload: {}) => callAPI({:?}, {:?}, payload) as Promise<AsyncData<{}, FetchError>>,\n",
                        method,
                        payload,
                        method,
                        path,
                        returns.clone().unwrap_or("void".into()),
                    )
                } else {
                    format!(
                        "{}: () => callAPI({:?}, {:?}) as Promise<AsyncData<{}, FetchError>>,\n",
                        method,
                        method,
                        path,
                        returns.clone().unwrap_or("void".into()),
                    )
                }
            }
            EntryNode::Path { slug, children } => {
                let inner = children
                    .iter()
                    .map(|c| c.gen_code(path.clone() + "/" + slug))
                    .reduce(|acc, cur| acc + &cur)
                    .unwrap();
                format!("{}: {{ {} }},\n", slug, inner)
            }
        }
    }
}

/// 添加 overload function declaration
fn gen_entry(service: server::ServiceDoc) -> EntryNode {
    let mut children = Vec::new();
    for api in service.apis {
        // some invalid case
        if api.query_type.is_some() && api.body_type.is_some() {
            panic!("query conflict with body payload")
        }
        if api.query_type.is_some() && api.method != "get" {
            panic!("query should not by used for non-get api")
        }
        if api.method == "get" && api.body_type.is_some() {
            panic!("body should not by used for get api")
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
                payload: api.body_type.or(api.query_type),
                returns: api.res_type,
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
        headers: useRequestHeaders(['cookie'])
    };
    if (args === undefined) {
        return useFetch(path, options);
    } else if (method === 'get') {
        return useFetch(path, { ...options, query: args });
    } else {
        return useFetch(path, { ...options, body: args });
    }
}
"#
    .into()
}

fn main() {
    let entry = EntryRoot(vec![
        gen_entry(app::auth::service_doc()),
        gen_entry(app::user::service_doc()),
    ]);

    let code = String::from(
        r#"// generated by server/src/bin/gen_docs.rs
// DO NOT EDIT.

import type { AsyncData } from "nuxt/app";
import type { FetchError } from "ofetch";

"#,
    ) + &gen_nuxt_basic()
        + &entry.gen_code();
    println!("{code}");
}
