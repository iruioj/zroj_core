#![allow(dead_code)]
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

#[derive(Serialize, SerdeJsonWithType)]
enum EnumRenameTest {
    #[serde(rename = "hello")]
    Hello,
    World(i32, Vec<String>),
    #[serde(rename = "less")]
    More {
        #[serde(rename = "good")]
        name: String,
        id: usize,
    },
}

#[test]
fn test_serde() {
    assert_eq!(Test1::type_def(), "{name: string;tags: [string,number][];}");
    assert_eq!(TestGenerics::<u32>::type_def(), "{name: number;}");
    assert_eq!(Unnamed::type_def(), "[string,boolean,number]");
    assert_eq!(Single::type_def(), "boolean");
    assert_eq!(
        EnumTest::type_def(),
        "\"Hello\" | {World: [number,string[]]} | {More: {name: string;id: number;}}"
    );
    assert_eq!(Discriminate::type_def(), "\"A\"");
    assert_eq!(
        EnumRenameTest::type_def(),
        "\"hello\" | {World: [number,string[]]} | {less: {good: string;id: number;}}"
    );
}
