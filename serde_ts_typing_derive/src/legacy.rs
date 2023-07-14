// /// 对于带有泛型的类型，我们要求所有类型参数都实现 TypeDef
// pub fn gen_where_clause(generics: Generics) -> Option<proc_macro2::TokenStream> {
//     generics
//         .params
//         .iter()
//         .filter_map(|e| match e {
//             GenericParam::Type(x) => Some(x),
//             _ => None,
//         })
//         // 要求所有 generic type 都实现 TypeDef
//         .map(|e| {
//             let ident = &e.ident;
//             quote!( #ident: serde_ts_typing::TypeDef, )
//         })
//         .reduce(|mut acc, e| {
//             acc.extend(e);
//             acc
//         })
//         .map(|bounds| {
//             if let Some(c) = generics.where_clause {
//                 let p = c.predicates;
//                 quote!(where #bounds #p )
//             } else {
//                 quote!(where #bounds)
//             }
//         })
// }