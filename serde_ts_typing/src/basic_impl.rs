use super::*;

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

macro_rules! impl_scalar_cf {
    ($( $type:ty )*, $v:path) => {
        $(
            impl SerdeJsonTsType for $type {
                fn type_def() -> TypeExpr {
                    $v
                }
            }
        )*
    };
}

impl_scalar_cf!(String, TypeExpr::String);
impl_scalar_cf!(i16 u16 i32 u32 i64 u64 isize usize f32 f64, TypeExpr::Number);
impl_scalar_cf!(bool, TypeExpr::Boolean);

impl<T: SerdeJsonTsType> SerdeJsonTsType for Vec<T> {
    fn type_context() -> BTreeMap<String, TypeExpr> {
        T::type_context()
    }
    fn type_def() -> TypeExpr {
        TypeExpr::Array(Box::new(T::type_def()))
    }
}

macro_rules! impl_tuple_cf {
    ( $( $type:ident ),*  ) => {
        impl< $( $type : SerdeJsonTsType, )* > SerdeJsonTsType for ( $( $type, )* ) {
            // merge context
            fn type_context() -> BTreeMap<String, TypeExpr> {
                let mut ctx = BTreeMap::new();
                $( ctx.extend($type::type_context().into_iter()); )*
                ctx
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
