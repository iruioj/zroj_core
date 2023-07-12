mod basic_impl;
mod value;

use std::collections::{BTreeMap, BTreeSet};
pub use value::Value;

pub enum Error {
    /// identifier occurs in context free type expression
    CtxFreeTypeExprInvalidIdent(String),
}

/// TypeScript type representation
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeExpr {
    /// name of another type
    Ident(String),
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
}

impl ToString for TypeExpr {
    fn to_string(&self) -> String {
        match self {
            TypeExpr::Ident(i) => i.clone(),
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
        }
    }
}

// 不需要上下文的类型定义（非递归，不含有未知类型的 identifier）
// pub struct CtxFreeTypeExpr(TypeExpr);

// impl From<CtxFreeTypeExpr> for TypeExpr {
//     fn from(value: CtxFreeTypeExpr) -> Self {
//         value.0
//     }
// }

// impl TryFrom<TypeExpr> for CtxFreeTypeExpr {
//     type Error = Error;

//     fn try_from(value: TypeExpr) -> Result<Self, Self::Error> {
//         use TypeExpr::*;
//         match value {
//             Ident(s) => Err(Error::CtxFreeTypeExprInvalidIdent(s)),
//             Array(e) => Ok(Self(Array(Box::new(
//                 ((*e).try_into() as Result<CtxFreeTypeExpr, _>)?.into(),
//             )))),
//             Record(r) => Ok(Self(Record(
//                 r.into_iter()
//                     .map(|(k, v)| Ok((k, (v.try_into() as Result<CtxFreeTypeExpr, _>)?.into())))
//                     .collect::<Result<BTreeMap<std::string::String, TypeExpr>, Error>>()?,
//             ))),
//             Union(r) => Ok(Self(Union(
//                 r.into_iter()
//                     .map(|v| Ok((v.try_into() as Result<CtxFreeTypeExpr, _>)?.into()))
//                     .collect::<Result<BTreeSet<TypeExpr>, Error>>()?,
//             ))),
//             Tuple(r) => Ok(Self(Tuple(
//                 r.into_iter()
//                     .map(|v| Ok((v.try_into() as Result<CtxFreeTypeExpr, _>)?.into()))
//                     .collect::<Result<Vec<TypeExpr>, Error>>()?,
//             ))),

//             Value(_) | String | Number | Boolean => Ok(Self(value)),
//         }
//     }
// }

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

pub trait SerdeJsonTsType {
    fn type_context() -> BTreeMap<String, TypeExpr> {
        // no context
        Default::default()
    }
    fn type_def() -> TypeExpr;
}

#[allow(unused_imports)]
#[macro_use]
extern crate serde_ts_typing_derive;

pub use serde_ts_typing_derive::SerdeJsonWithType;
