mod basic_impl;
mod value;

use std::collections::{BTreeMap, BTreeSet};
pub use value::Value;

pub enum Error {
    /// identifier occurs in context free type expression
    CtxFreeTypeExprInvalidIdent(String),
}

/// TypeScript type representation
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
    Record(BTreeMap<String, TypeExpr>),
    /// `[T, S]`
    Tuple(Vec<TypeExpr>),
    /// ` T | S`
    Union(BTreeSet<TypeExpr>),
    /// ` T & S`
    Intersection(BTreeSet<TypeExpr>),
    /// any
    Any,
}

impl ToString for TypeExpr {
    fn to_string(&self) -> String {
        match self {
            TypeExpr::Undefined => "undefined".into(),
            TypeExpr::Ident(_, n) => n.clone(),
            TypeExpr::Value(v) => v.to_string(),
            TypeExpr::String => "string".into(),
            TypeExpr::Number => "number".into(),
            TypeExpr::Boolean => "boolean".into(),
            TypeExpr::Array(t) => t.to_string() + "[]",
            TypeExpr::Record(t) => {
                t.iter().fold(String::from("{"), |acc, (k, v)| {
                    format!("{acc}{k}:{};", v.to_string())
                }) + "}"
            }
            TypeExpr::Tuple(t) => {
                let mut sep = "";
                t.iter().fold(String::from("["), |acc, v| {
                    let r = format!("{acc}{sep}{}", v.to_string());
                    sep = ",";
                    r
                }) + "]"
            }
            TypeExpr::Union(t) => {
                let mut sep = "";
                t.iter().fold(String::from("("), |acc, v| {
                    let r = format!("{acc}{sep}{}", v.to_string());
                    sep = "|";
                    r
                }) + ")"
            }
            TypeExpr::Intersection(t) => {
                let mut sep = "";
                t.iter().fold(String::from("("), |acc, v| {
                    let r = format!("{acc}{sep}{}", v.to_string());
                    sep = "&";
                    r
                }) + ")"
            }
            TypeExpr::Any => "any".into()
        }
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
pub struct Context(BTreeMap<String, TypeExpr>, BTreeMap<TypeId, String>);

impl From<(BTreeMap<String, TypeExpr>, BTreeMap<TypeId, String>)> for Context {
    fn from(value: (BTreeMap<String, TypeExpr>, BTreeMap<TypeId, String>)) -> Self {
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
    pub fn register(&mut self, ty: TypeId, name: String, tydef: TypeExpr) {
        self.0.insert(name.clone(), tydef);
        self.1.insert(ty, name);
    }
    pub fn register_variant(&mut self, name: String, tydef: TypeExpr) {
        self.0.insert(name, tydef);
    }
    pub fn contains(&self, id: TypeId) -> bool {
        self.1.contains_key(&id)
    }
    pub fn render_code(&self) -> String {
        let mut r = String::new();
        for (name, tydef) in &self.0 {
            r += &format!("export type {name} = {};\n", tydef.to_string());
        }
        r
    }
    pub fn get_ty_by_id(&self, id: &TypeId) -> Option<&TypeExpr> {
        self.0.get(self.1.get(id)?)
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
