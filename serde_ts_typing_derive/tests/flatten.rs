use serde::Serialize;
use serde_ts_typing::{TsType, TypeExpr};

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
