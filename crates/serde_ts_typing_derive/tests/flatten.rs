use serde::{Deserialize, Serialize};
use serde_ts_typing::{TsType, TypeExpr};
use serde_with::{serde_as, DisplayFromStr};

#[derive(Serialize, TsType)]
#[ts(inline)]
struct A {
    name: String,
    // can only flatten structs and maps
    // note that flatten must be used with `ts(inline)`
    #[serde(flatten)]
    data: B,
}

#[derive(Serialize, TsType)]
#[ts(inline)]
struct B {
    id: u8,
    flag: bool,
}
#[test]
fn test_flatten() {
    let v = A {
        name: "hello".into(),
        data: B { id: 1, flag: true },
    };
    eprintln!("{}", serde_json::to_string_pretty(&v).unwrap());
    assert_eq!(
        A::type_def(),
        TypeExpr::Struct(
            [
                ("name".into(), TypeExpr::String),
                ("id".into(), TypeExpr::Number),
                ("flag".into(), TypeExpr::Boolean),
            ]
            .into_iter()
            .collect()
        )
    )
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, TsType)]
#[ts(inline)]
struct ListQuery {
    /// 利用类型限制，一次请求的数量不能超过 256 个
    #[serde_as(as = "DisplayFromStr")]
    #[ts(as_type = "String")]
    max_count: u8,

    /// 跳过前 offset 个结果
    #[serde_as(as = "DisplayFromStr")]
    #[ts(as_type = "String")]
    offset: u32,
}

#[derive(Debug, Serialize, Deserialize, TsType)]
#[ts(inline)]
struct ProbMetasQuery {
    #[serde(flatten)]
    list: ListQuery,
    // /// 利用类型限制，一次请求的数量不能超过 256 个
    // max_count: u8,

    // /// 跳过前 offset 个结果
    // offset: u32,
    /// 搜索的关键字/模式匹配
    pattern: Option<String>,
}

#[test]
fn test_serde_url() {
    let query: ProbMetasQuery = serde_urlencoded::from_str("max_count=5&offset=6").unwrap();
    dbg!(&query);
    eprintln!("{}", serde_json::to_string_pretty(&query).unwrap());
    eprintln!("{}", ProbMetasQuery::type_def());
}
