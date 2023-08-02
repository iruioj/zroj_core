use std::any::TypeId;

use serde::Serialize;
use serde_ts_typing::{TsType, TypeExpr};

#[derive(Serialize, TsType)]
#[ts(inline)]
struct NewTypeStruct(Vec<String>);

#[derive(Serialize, TsType, Default)]
struct TupleStruct(i8, usize, bool);

#[derive(Serialize, TsType, Default)]
#[ts(inline)]
struct UnitStruct;

#[derive(Serialize, TsType, Default)]
#[ts(inline)]
struct SimpleStruct {
    hello: bool,
    world: String,
    tuple: TupleStruct,
}

#[derive(Serialize, TsType, Default)]
// #[serde(tag = "...")] can only be used on enums and structs with named fields
#[serde(tag = "tt", rename = "ggg")]
#[ts(inline)]
struct TagSimpleStruct {
    hello: bool,
}

#[test]
fn test_struct() {
    let v = NewTypeStruct(vec!["hello".into(), "world".into()]);
    assert_eq!(
        NewTypeStruct::type_def(),
        TypeExpr::Array(Box::new(TypeExpr::String))
    );
    assert_eq!(r#"["hello","world"]"#, serde_json::to_string(&v).unwrap());

    let v = TupleStruct::default();
    assert_eq!(
        TupleStruct::type_context()
            .get_ty_by_id(&TypeId::of::<TupleStruct>())
            .unwrap()
            .clone(),
        TypeExpr::Tuple(vec![TypeExpr::Number, TypeExpr::Number, TypeExpr::Boolean])
    );
    assert!(matches!(TupleStruct::type_def(), TypeExpr::Ident(_, _)));
    assert_eq!(r#"[0,0,false]"#, serde_json::to_string(&v).unwrap());

    let v = UnitStruct::default();
    assert_eq!(
        UnitStruct::type_def(),
        TypeExpr::Value(serde_ts_typing::Value::Null)
    );
    assert_eq!("null", serde_json::to_string(&v).unwrap());

    let v = SimpleStruct::default();
    assert_eq!(
        SimpleStruct::type_def(),
        TypeExpr::Record(
            [
                ("hello".into(), TypeExpr::Boolean),
                ("world".into(), TypeExpr::String),
                ("tuple".into(), TupleStruct::type_def())
            ]
            .into_iter()
            .collect()
        )
    );
    assert_eq!(
        "{\"hello\":false,\"world\":\"\",\"tuple\":[0,0,false]}",
        serde_json::to_string(&v).unwrap()
    );

    let v = TagSimpleStruct::default();
    assert_eq!(
        TagSimpleStruct::type_def(),
        TypeExpr::Record(
            [
                ("hello".into(), TypeExpr::Boolean),
                (
                    "tt".into(),
                    TypeExpr::Value(serde_ts_typing::Value::String("ggg".into()))
                ),
            ]
            .into_iter()
            .collect()
        )
    );
    assert_eq!(
        "{\"tt\":\"ggg\",\"hello\":false}",
        serde_json::to_string(&v).unwrap()
    );
}

#[derive(Serialize, TsType, Default)]
#[ts(inline)]
struct StructWithOption {
    hello: Option<String>,
}

#[test]
fn test_option() {
    let v1: Option<String> = Some("hello".to_owned());
    let v2: Option<String> = None;
    assert_eq!(
        <Option<String> as TsType>::type_def(),
        TypeExpr::Union(
            [
                TypeExpr::Value(serde_ts_typing::Value::Null),
                TypeExpr::String
            ]
            .into_iter()
            .collect()
        )
    );
    dbg!(serde_json::to_string(&v1).unwrap());
    dbg!(serde_json::to_string(&v2).unwrap());

    let v = StructWithOption::default();
    assert_eq!("{\"hello\":null}", serde_json::to_string(&v).unwrap());
}

#[derive(Serialize, TsType)]
struct Recursive {
    id: u32,
    is_key: bool,
    children: Vec<Recursive>,
}

#[derive(Serialize, TsType)]
#[allow(dead_code)]
enum Node {
    Root(Root),
    ShadowRoot(ShadowRoot),
}

#[derive(Serialize, TsType)]
struct Root {
    children: Vec<Node>,
}

#[derive(Serialize, TsType)]
struct ShadowRoot {
    id: String,
    root: Root,
}

#[test]
fn test_recursive() {
    dbg!(Recursive::type_def());
    dbg!(Recursive::type_context());
    dbg!(Node::type_def());
    eprintln!("{}", Node::type_context().render_code());
}
