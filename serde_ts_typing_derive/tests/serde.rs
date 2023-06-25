use serde::Serialize;
use serde_ts_typing::{SerdeJsonWithType, TypeDef};

#[derive(Serialize, SerdeJsonWithType)]
struct Test1 {
    name: String,
    tags: Vec<(String, u32)>,
}

#[derive(Serialize, SerdeJsonWithType)]
struct TestGenerics<T>
where
    T: Eq,
{
    name: T,
}

#[derive(Serialize, SerdeJsonWithType)]
struct Unnamed(String, bool, i32);

#[derive(Serialize, SerdeJsonWithType)]
struct Single(bool);

#[test]
fn test_serde() {
    println!(
        "{}",
        serde_json::to_string_pretty(&Test1 {
            name: "hello".into(),
            tags: vec![("world".into(), 5)]
        })
        .unwrap()
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&Unnamed("aa".into(), false, 10))
        .unwrap()
    );
    dbg!(Test1::type_def());
    dbg!(TestGenerics::<u32>::type_def());
    dbg!(Unnamed::type_def());
    dbg!(serde_json::to_string_pretty(&Single(false)).unwrap());
    dbg!(Single::type_def());
}
