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

#[derive(Serialize, SerdeJsonWithType)]
enum EnumTest {
    Hello,
    World(i32, Vec<String>),
    More { name: String, id: usize },
}

#[derive(Serialize, SerdeJsonWithType)]
enum Discriminate {
    A = 0,
}

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
        serde_json::to_string_pretty(&Unnamed("aa".into(), false, 10)).unwrap()
    );
    dbg!(Test1::type_def());
    dbg!(TestGenerics::<u32>::type_def());
    dbg!(Unnamed::type_def());
    dbg!(serde_json::to_string_pretty(&Single(false)).unwrap());
    dbg!(Single::type_def());
    dbg!(serde_json::to_string(&EnumTest::Hello).unwrap());
    dbg!(serde_json::to_string(&EnumTest::World(0, Vec::new())).unwrap());
    dbg!(serde_json::to_string(&EnumTest::More {
        name: "bbb".into(),
        id: 12
    }))
    .unwrap();
    dbg!(EnumTest::type_def());
    dbg!(serde_json::to_string(&Discriminate::A).unwrap());
    dbg!(Discriminate::type_def());
}
