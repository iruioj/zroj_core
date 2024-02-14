mod basic_impl;
mod value;

use askama::Template;
use std::collections::{BTreeMap, BTreeSet};
pub use value::Value;

pub enum Error {
    /// identifier occurs in context free type expression
    CtxFreeTypeExprInvalidIdent(String),
}

/// TypeScript type representation.
///
/// ```rust
/// # use serde_ts_typing::TypeExpr;
/// let t = TypeExpr::Struct([
///     (String::from("name"), TypeExpr::String),
///     (String::from("permissions"), TypeExpr::Array(Box::new(TypeExpr::Number))),
/// ].into());
///
/// // print the type with 80 line width limit and 4-space indent
/// eprintln!("{:80.4}", t);
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TypeExpr {
    /// `undefined`
    Undefined,
    /// name, id of another type
    /// `type name = {...}`
    Ident(TypeId, String),
    /// 具体的 value，例如 `type Name = 'hello';`
    Value(value::Value),
    /// `string`
    String,
    /// `number`
    Number,
    /// `boolean`
    Boolean,
    /// `T[]`
    Array(Box<TypeExpr>),
    /// `{ a: T, b: S }`
    Struct(BTreeMap<String, TypeExpr>),
    /// `Record<K, V>`
    Record(Box<TypeExpr>, Box<TypeExpr>),
    /// `[T, S]`
    Tuple(Vec<TypeExpr>),
    /// ` T | S`
    Union(BTreeSet<TypeExpr>),
    /// ` T & S`
    Intersection(BTreeSet<TypeExpr>),
    /// any
    Any,
}

impl std::fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = f.precision().unwrap_or(0);
        let line_width = f.width().unwrap_or(0);
        f.write_str(&self.print_linewidth(indent, line_width))
    }
}

#[derive(Template)]
#[template(
    source = "
{%- if indent > 0 -%}

{
{% for (k, v) in record -%}
{{ \" \".repeat(indent.clone()) }}{{ k }}: {{ v|indent(indent.clone()) }};
{% endfor -%}
}

{%- else -%}

{ {%~ for (k, v) in record -%} {{ k }}: {{ v }}; {%~ endfor -%} }

{%- endif -%}",
    ext = "txt"
)]
struct StructTemplate<'a> {
    record: BTreeMap<&'a String, String>,
    indent: usize,
}

#[derive(Template)]
#[template(
    source = "
{%- if indent > 0 -%}

[
{% for v in items -%}
{{ \" \".repeat(indent.clone()) }}{{ v|indent(indent.clone()) }},
{% endfor -%}
]

{%- else -%}

[ {{ items.join(\", \") }} ]

{%- endif -%}",
    ext = "txt"
)]
struct TupleTemplate {
    items: Vec<String>,
    indent: usize,
}

#[derive(Template)]
#[template(
    source = "
{%- if indent > 0 -%}

(
{{ \" \".repeat(indent.clone()) }}{{ items[0]|indent(indent.clone()) }}
{% for v in items.iter().skip(1) -%}
{{ \" \".repeat(indent.clone()) }}| {{ v|indent(indent.clone()) }}
{% endfor -%}
)

{%- else -%}

( {{ items.join(\" | \") }} )

{%- endif -%}",
    ext = "txt"
)]
struct UnionTemplate {
    items: Vec<String>,
    indent: usize,
}

#[derive(Template)]
#[template(
    source = "
{%- if indent > 0 -%}

(
{{ \" \".repeat(indent.clone()) }}{{ items[0]|indent(indent.clone()) }}
{% for v in items.iter().skip(1) -%}
{{ \" \".repeat(indent.clone()) }}& {{ v|indent(indent.clone()) }}
{% endfor -%}
)

{%- else -%}

( {{ items.join(\" & \") }} )

{%- endif -%}",
    ext = "txt"
)]
struct SectTemplate {
    items: Vec<String>,
    indent: usize,
}

impl TypeExpr {
    /// Print the type expression w/o indent, trying to obey line width limitation.
    ///
    /// If `indent == 0`, print expression in one line.
    /// Otherwise, print expression with indent and WITHOUT ending new line.
    fn print_linewidth(&self, indent: usize, line_width: usize) -> String {
        let lw2 = line_width.saturating_sub(4);
        match self {
            // cases not care about indent
            TypeExpr::Undefined => "undefined".into(),
            TypeExpr::String => "string".into(),
            TypeExpr::Number => "number".into(),
            TypeExpr::Boolean => "boolean".into(),
            TypeExpr::Any => "any".into(),
            TypeExpr::Ident(_, n) => n.clone(),
            TypeExpr::Value(v) => v.to_string(),

            // cases not increase indent
            TypeExpr::Array(t) => {
                let oneline = t.print_linewidth(0, line_width) + "[]";
                if oneline.len() < line_width || indent == 0 {
                    oneline
                } else {
                    t.print_linewidth(indent, line_width) + "[]"
                }
            }
            TypeExpr::Record(k, v) => format!(
                "Record<{}, {}>",
                k.print_linewidth(indent, line_width),
                v.print_linewidth(indent, line_width)
            ),

            // cases formatted by templates
            TypeExpr::Struct(t) => {
                let bs = t
                    .iter()
                    .map(|(k, v)| (k, v.print_linewidth(indent, lw2)))
                    .collect();
                let tp = StructTemplate { record: bs, indent };
                tp.render().unwrap()
            }
            TypeExpr::Tuple(t) => {
                let oneline = TupleTemplate {
                    items: t.iter().map(|v| v.print_linewidth(0, line_width)).collect(),
                    indent: 0,
                }
                .render()
                .unwrap();
                if oneline.len() < line_width || indent == 0 {
                    oneline
                } else {
                    let items = t.iter().map(|v| v.print_linewidth(indent, lw2)).collect();
                    let tp = TupleTemplate { items, indent };
                    tp.render().unwrap()
                }
            }
            TypeExpr::Union(t) => {
                let oneline = UnionTemplate {
                    items: t.iter().map(|v| v.print_linewidth(0, line_width)).collect(),
                    indent: 0,
                }
                .render()
                .unwrap();
                if oneline.len() < line_width || indent == 0 {
                    oneline
                } else {
                    let items = t.iter().map(|v| v.print_linewidth(indent, lw2)).collect();
                    let tp = UnionTemplate { items, indent };
                    tp.render().unwrap()
                }
            }
            TypeExpr::Intersection(t) => {
                let oneline = SectTemplate {
                    items: t.iter().map(|v| v.print_linewidth(0, line_width)).collect(),
                    indent: 0,
                }
                .render()
                .unwrap();
                if oneline.len() < line_width || indent == 0 {
                    oneline
                } else {
                    let items = t.iter().map(|v| v.print_linewidth(indent, lw2)).collect();
                    let tp = SectTemplate { items, indent };
                    tp.render().unwrap()
                }
            }
        }
    }
}

impl TypeExpr {
    pub fn new_struct() -> Self {
        TypeExpr::Struct(BTreeMap::new())
    }
    pub fn struct_insert(&mut self, k: String, v_type: TypeExpr) {
        let Self::Struct(s) = self else {
            panic!("invalid struct to insert")
        };
        s.insert(k, v_type);
    }
    pub fn struct_merge(&mut self, v_type: TypeExpr) {
        let Self::Struct(s) = self else {
            panic!("invalid struct to merge")
        };
        let Self::Struct(t) = v_type else {
            panic!("invalid struct to be merged {:?}", v_type)
        };
        t.into_iter().for_each(|(k, v)| {
            s.insert(k, v);
        });
    }
}

/// 对于 Rust 类型提供 typescript 类型生成
pub trait TypeDef {
    /// 生成 typescript 类型
    fn type_def() -> String;
}

/// 一个 marker trait
///
/// 为实现了 Serialize 的类型提供 typescript 类型生成
pub trait SerdeJsonWithType
where
    Self: TypeDef,
{
}

pub type TypeId = std::any::TypeId;

/// 类型标志符的上下文（类型集合）
#[derive(Debug, Default)]
pub struct Context(
    BTreeMap<String, (TypeExpr, String)>,
    BTreeMap<TypeId, String>,
);

impl
    From<(
        BTreeMap<String, (TypeExpr, String)>,
        BTreeMap<TypeId, String>,
    )> for Context
{
    fn from(
        value: (
            BTreeMap<String, (TypeExpr, String)>,
            BTreeMap<TypeId, String>,
        ),
    ) -> Self {
        Self(value.0, value.1)
    }
}

impl std::ops::Add for Context {
    type Output = Context;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        rhs.0.append(&mut self.0);
        rhs.1.append(&mut self.1);
        rhs
    }
}

impl Context {
    pub fn register(&mut self, ty: TypeId, name: String, tydef: TypeExpr, docs: String) {
        self.0.insert(name.clone(), (tydef, docs));
        self.1.insert(ty, name);
    }
    pub fn register_variant(&mut self, name: String, tydef: TypeExpr, docs: String) {
        self.0.insert(name, (tydef, docs));
    }
    pub fn contains(&self, id: TypeId) -> bool {
        self.1.contains_key(&id)
    }
    /// rendering all types as exported type
    pub fn render_code(&self, indent: usize) -> String {
        let mut r = String::new();
        for (name, (tydef, docs)) in &self.0 {
            r += &format!(
                "/**\n{docs}*/\nexport type {name} = {:80.indent$};\n",
                tydef
            );
        }
        r
    }
    pub fn get_ty_by_id(&self, id: &TypeId) -> Option<&TypeExpr> {
        self.0.get(self.1.get(id)?).map(|o| &o.0)
    }
}

pub trait TsType
where
    Self: 'static,
{
    fn register_context(c: &mut Context);
    fn type_def() -> TypeExpr;

    /// 如果 context 中不包含自身的 context 就调用 register_context
    fn register_self_context(c: &mut Context) {
        if !c.contains(TypeId::of::<Self>()) {
            Self::register_context(c)
        }
    }
    fn type_context() -> Context {
        let mut r = Context::default();
        Self::register_context(&mut r);
        r
    }
}

#[allow(unused_imports)]
#[macro_use]
extern crate serde_ts_typing_derive;

pub use serde_ts_typing_derive::TsType;

#[cfg(test)]
mod tests {
    use crate::TypeExpr;

    #[test]
    fn test_typeexpr() {
        let person = TypeExpr::Struct(
            [
                (String::from("name"), TypeExpr::String),
                (
                    String::from("permissions"),
                    TypeExpr::Array(Box::new(TypeExpr::Number)),
                ),
            ]
            .into(),
        );
        assert_eq!(
            person.print_linewidth(0, 0),
            "{ name: string; permissions: number[]; }"
        );
        assert_eq!(
            person.print_linewidth(4, 0),
            "{\n    name: string;\n    permissions: number[];\n}"
        );

        let person_ext = TypeExpr::Struct(
            [
                (String::from("person"), person),
                (String::from("age"), TypeExpr::Number),
                (String::from("zoo"), TypeExpr::Number),
            ]
            .into(),
        );
        assert_eq!(
            person_ext.print_linewidth(4, 0),
            r#"{
    age: number;
    person: {
        name: string;
        permissions: number[];
    };
    zoo: number;
}"#
        );

        let person_map = TypeExpr::Record(Box::new(TypeExpr::String), Box::new(person_ext.clone()));
        assert_eq!(
            person_map.print_linewidth(4, 0),
            r#"Record<string, {
    age: number;
    person: {
        name: string;
        permissions: number[];
    };
    zoo: number;
}>"#
        );

        let tuple = TypeExpr::Tuple(vec![
            TypeExpr::Number,
            person_ext.clone(),
            person_map.clone(),
            TypeExpr::String,
        ]);

        assert_eq!(
            r#"[
    number,
    {
        age: number;
        person: {
            name: string;
            permissions: number[];
        };
        zoo: number;
    },
    Record<string, {
        age: number;
        person: {
            name: string;
            permissions: number[];
        };
        zoo: number;
    }>,
    string,
]"#,
            tuple.print_linewidth(4, 0)
        );

        let union = TypeExpr::Union(
            [
                TypeExpr::Number,
                person_ext.clone(),
                person_map.clone(),
                TypeExpr::String,
            ]
            .into(),
        );
        assert_eq!(
            r#"(
    string
    | number
    | {
        age: number;
        person: {
            name: string;
            permissions: number[];
        };
        zoo: number;
    }
    | Record<string, {
        age: number;
        person: {
            name: string;
            permissions: number[];
        };
        zoo: number;
    }>
)"#,
            union.print_linewidth(4, 0)
        );

        let sect = TypeExpr::Intersection(
            [
                TypeExpr::Number,
                person_ext.clone(),
                person_map.clone(),
                TypeExpr::String,
            ]
            .into(),
        );

        assert_eq!(
            r#"(
    string
    & number
    & {
        age: number;
        person: {
            name: string;
            permissions: number[];
        };
        zoo: number;
    }
    & Record<string, {
        age: number;
        person: {
            name: string;
            permissions: number[];
        };
        zoo: number;
    }>
)"#,
            sect.print_linewidth(4, 0)
        );

        let sect = TypeExpr::Intersection([TypeExpr::Number, TypeExpr::String].into());
        eprintln!("{:80.4}", sect)
    }
}
