use serde::Serialize;
use serde_ts_typing::{TsType, TypeExpr};

#[derive(Serialize, TsType)]
#[ts(inline)]
enum MixedEnum {
    A { hello: bool },
    B(String, bool),
    C(u8),
    D,
}

#[derive(Serialize, TsType)]
#[ts(inline)]
struct Test{
    flag: bool,
}

#[derive(Serialize, TsType)]
#[ts(inline)]
#[serde(tag = "tt")]
#[allow(dead_code)]
enum TestEnum {
    Test(Test)
}

#[derive(Serialize, TsType)]
#[ts(inline)]
#[serde(tag = "tt")]
enum NewTypeVariantEnum {
    Mixed(MixedEnum)
}

#[test]
fn test_enum() {
    dbg!(serde_json::to_string(&MixedEnum::A { hello: true }).unwrap());
    dbg!(serde_json::to_string(&MixedEnum::B("ok".into(), true)).unwrap());
    dbg!(serde_json::to_string(&MixedEnum::C(255)).unwrap());
    dbg!(serde_json::to_string(&MixedEnum::D).unwrap());
    assert_eq!(
        MixedEnum::type_def(),
        TypeExpr::Union(
            [
                TypeExpr::Value(serde_ts_typing::Value::String("D".into())),
                TypeExpr::Record(
                    [(
                        "A".into(),
                        TypeExpr::Record(
                            [("hello".into(), TypeExpr::Boolean)].into_iter().collect()
                        )
                    )]
                    .into_iter()
                    .collect()
                ),
                TypeExpr::Record(
                    [(
                        "B".into(),
                        TypeExpr::Tuple(vec![TypeExpr::String, TypeExpr::Boolean])
                    )]
                    .into_iter()
                    .collect()
                ),
                TypeExpr::Record([("C".into(), TypeExpr::Number)].into_iter().collect())
            ]
            .into_iter()
            .collect()
        )
    );

    eprintln!("{}", serde_json::to_string(&NewTypeVariantEnum::Mixed(MixedEnum::D)).unwrap());
    eprintln!("{}", serde_json::to_string(&MixedEnum::D).unwrap());
    eprintln!("{}", TestEnum::type_def().to_string());
}