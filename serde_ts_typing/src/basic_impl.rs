use super::*;

macro_rules! impl_scalar_cf {
    ($( $type:ty )*, $v:path) => {
        $(
            impl TsType for $type {
                fn register_context(_: &mut Context) {
                    // nothing
                }
                fn type_def() -> TypeExpr {
                    $v
                }
            }
        )*
    };
}

impl_scalar_cf!(String, TypeExpr::String);
impl_scalar_cf!(i8 u8 i16 u16 i32 u32 i64 u64 isize usize f32 f64, TypeExpr::Number);
impl_scalar_cf!(bool, TypeExpr::Boolean);

impl<T: TsType> TsType for Vec<T> {
    fn register_context(c: &mut Context) {
        T::register_self_context(c);
    }
    fn type_def() -> TypeExpr {
        TypeExpr::Array(Box::new(T::type_def()))
    }
}

macro_rules! impl_tuple_cf {
    ( $( $type:ident ),*  ) => {
        impl< $( $type : TsType, )* > TsType for ( $( $type, )* ) {
            // merge context
            fn register_context(c: &mut Context) {
                $( $type::register_self_context(c); )*
            }
            fn type_def() -> TypeExpr {
                let vs = vec![ $( $type::type_def().into() ),* ];
                TypeExpr::Tuple(vs)
            }
        }
    };
}

impl_tuple_cf!(A);
impl_tuple_cf!(A, B);
impl_tuple_cf!(A, B, C);
impl_tuple_cf!(A, B, C, D);
impl_tuple_cf!(A, B, C, D, E);
impl_tuple_cf!(A, B, C, D, E, F);
impl_tuple_cf!(A, B, C, D, E, F, G);

impl<T: TsType> TsType for Option<T> {
    fn register_context(c: &mut Context) {
        T::register_self_context(c);
    }
    fn type_def() -> TypeExpr {
        TypeExpr::Union(
            [TypeExpr::Value(Value::Null), T::type_def()]
                .into_iter()
                .collect(),
        )
    }
}

impl<T: TsType> TsType for Box<T> {
    fn register_context(c: &mut Context) {
        T::register_self_context(c);
    }
    fn type_def() -> TypeExpr {
        T::type_def()
    }
}
