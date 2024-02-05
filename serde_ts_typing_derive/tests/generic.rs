use serde::Serialize;
use serde_ts_typing::TsType;

#[derive(Serialize, TsType)]
struct Unit;

#[derive(Serialize, TsType)]
#[ts(inline)]
struct NewTypeStruct<T: Serialize>(T);

#[test]
fn test_generics() {
    let t = NewTypeStruct(Unit);
    let r = serde_json::to_string(&t).unwrap();
    dbg!(r);
}
