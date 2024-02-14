//! Backend-served page for API documents

use crate::marker::{AnyResult, QueryParam};
use actix_http::StatusCode;
use actix_web::HttpResponse;
use askama::Template;
use serde::Deserialize;
use serde_ts_typing::TsType;
use server_derive::{api, scope_service};

#[derive(Template)]
#[template(
    source = "<ul>
    {% for path in paths %}
        <li>
            <a href=\"?module={{ path }}\"><code>{{ path }}</code></a>
        </li>
    {% endfor %}
</ul>",
    ext = "txt"
)]
struct IndexTemplate {
    paths: Vec<String>,
}

#[derive(Template)]
#[template(
    source = "<h2>Document for <code>{{ root_path }}</code> <a href=\"?\">Back</a></h2>
    {% for api in apis %}
        {{ api }}
    {% endfor %}
<h3>Context Types</h3>
<pre>{{ context }}</pre>
    ",
    ext = "txt"
)]
struct ModuleTemplate {
    root_path: String,
    apis: Vec<String>,
    context: String,
}

#[derive(Template)]
#[template(
    source = "<div>
    <h3>{{ method|upper }}

    <code>{% if path.len() == 0 ~%}
        (index)
    {%~ else %}
        {{- path -}}
    {% endif %}</code>

    {% match query_type %}
    {% when Some with (q) %}
        <span>[Query]</span> {{ q }}
    {% when None %}
    {% endmatch %}

    {% match body_type %}
    {% when Some with (q) %}
        <span>[Body]</span> {{ q }}
    {% when None %}
    {% endmatch %}

    {% if is_form %}
        <span>[FormData]</span>
    {% endif %}

    {% match res_type %}
    {% when Some with (q) %}
        <span>[Response]</span> {{ q }}
    {% when None %}
    {% endmatch %}
    </h3>

    <div>{{ description }}</div>
</div>",
    ext = "txt"
)]
struct ApiTemplate {
    pub path: String,
    pub method: String,
    pub query_type: Option<String>,
    pub body_type: Option<String>,
    pub is_form: bool,
    pub res_type: Option<String>,
    pub description: String,
}

#[derive(Deserialize, TsType)]
struct DocsQuery {
    module: Option<String>,
}
#[api(method = get, path = "")]
async fn docs_get(query: QueryParam<DocsQuery>) -> AnyResult<HttpResponse> {
    let paths: Vec<String> = super::DOCS.iter().map(|t| t.0.path.clone()).collect();

    let html = if let Some(path) = query.0.module {
        let data = super::DOCS.iter().find(|t| t.0.path == path).unwrap();

        let apis: Vec<String> = data
            .0
            .apis
            .iter()
            .map(|api| {
                ApiTemplate {
                    path: api.path.clone(),
                    method: api.method.clone(),
                    query_type: api.query_type.as_ref().map(|e| e.to_string()),
                    body_type: api.body_type.as_ref().map(|e| e.to_string()),
                    is_form: api.is_form,
                    res_type: api.res_type.as_ref().map(|e| e.to_string()),
                    description: api.description.clone(),
                }
                .render()
                .unwrap()
            })
            .collect();

        ModuleTemplate {
            root_path: path,
            apis,
            context: data.1.render_code(4),
        }
        .render()
        .unwrap()
    } else {
        IndexTemplate { paths }.render().unwrap()
    };

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[scope_service(path = "/_docs")]
pub fn service() {
    service(docs_get);
}
