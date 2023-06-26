/// 对于 Rust 类型提供 typescript 类型生成
pub trait TypeDef {
    /// 生成 typescript 类型
    fn type_def() -> String;
}

macro_rules! impl_scalar {
    ($( $type:ty )*, $ts_type:literal) => {
        $(
            impl TypeDef for $type {
                fn type_def() -> String {
                    $ts_type.into()
                }
            }
        )*
    };
}

impl_scalar!(String str, "string");
impl_scalar!(i16 u16 i32 u32 i64 u64 isize usize f32 f64, "number");
impl_scalar!(bool, "boolean");

impl<T: TypeDef> TypeDef for Vec<T> {
    fn type_def() -> String {
        format!("{}[]", T::type_def())
    }
}

macro_rules! impl_tuple {
    ( $( $type:ident ),*  ) => {
        impl< $( $type : TypeDef, )* > TypeDef for ( $( $type, )* ) {
            fn type_def() -> String {
                let vs = vec![ $( $type::type_def() ),* ];
                String::from("[") + &vs.join(",") +"]"
            }
        }
    };
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);

impl<T: TypeDef> TypeDef for Option<T> {
    fn type_def() -> String {
        T::type_def() + " | undefined"
    }
}

/// 一个 marker trait
///
/// 为实现了 Serialize 的类型提供 typescript 类型生成
pub trait SerdeJsonWithType
where
    Self: TypeDef,
{
}

#[allow(unused_imports)]
#[macro_use]
extern crate serde_ts_typing_derive;
pub use serde_ts_typing_derive::SerdeJsonWithType;