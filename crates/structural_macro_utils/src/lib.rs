mod attrs;
mod fields;
mod generics;

pub use attrs::{AttrListVisitor, MetaList};
pub use fields::{FieldVisitor, FieldsVisitor};
pub use generics::GenericsVisitor;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

/// `'v` is the life time of the visitor
pub struct StructVisitor<'v>(pub &'v syn::ItemStruct);

impl<'v> StructVisitor<'v> {
    pub fn attrs(&self) -> AttrListVisitor<'_> {
        AttrListVisitor(&self.0.attrs)
    }
    pub fn generics(&self) -> GenericsVisitor<'_> {
        GenericsVisitor(&self.0.generics)
    }
    pub fn fields(&self) -> FieldsVisitor<'_> {
        FieldsVisitor(&self.0.fields)
    }
    pub fn ident(&self) -> &syn::Ident {
        &self.0.ident
    }
}

pub struct VarientVisitor<'v>(pub &'v syn::Variant);

impl<'v> VarientVisitor<'v> {
    pub fn attrs(&self) -> AttrListVisitor<'_> {
        AttrListVisitor(&self.0.attrs)
    }
    pub fn fields(&self) -> FieldsVisitor<'_> {
        FieldsVisitor(&self.0.fields)
    }
    pub fn ident(&self) -> &syn::Ident {
        &self.0.ident
    }
}

pub struct EnumVisitor<'v>(pub &'v syn::ItemEnum);

impl<'v> EnumVisitor<'v> {
    pub fn attrs(&self) -> AttrListVisitor<'_> {
        AttrListVisitor(&self.0.attrs)
    }
    pub fn generics(&self) -> GenericsVisitor<'_> {
        GenericsVisitor(&self.0.generics)
    }
    pub fn varients<'b>(
        &'b self,
    ) -> std::iter::Map<
        syn::punctuated::Iter<'b, syn::Variant>,
        fn(&'b syn::Variant) -> VarientVisitor<'b>,
    > {
        self.0.variants.iter().map(VarientVisitor)
    }
    pub fn ident(&self) -> &syn::Ident {
        &self.0.ident
    }
}

pub fn concat_token_stream(
    token_streams: Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    token_streams
        .into_iter()
        .reduce(|mut a, b| {
            a.extend(b);
            a
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::StructVisitor;

    #[doc = "name-value-meta"]
    #[deprecated]
    #[allow(dead_code)]
    struct TestStruct<
        'a,
        'b: 'a,
        'c,
        T,
        #[cfg(not(target_os = "linux"))] E: Eq,
        #[cfg(with_len)] const LEN: usize,
    >
    where
        T: std::fmt::Debug + Clone + 'a + ?Sized,
        &'a T: Default,
        'c: 'a,
    {
        #[doc = "inner"]
        inner: Box<T>,
        #[cfg(not(target_os = "linux"))]
        pub member: &'a E,
        pub(crate) name: &'a mut str,
        #[cfg(unix)]
        pub(super) id: &'b i32,
        #[cfg_attr(target_os = "windows", deny(lint))]
        pub(in crate::tests) admin: &'c bool,
    }

    #[allow(dead_code)]
    enum TestEnum {
        Good(Option<i32>),
        Bad {
            reason: String,
            trace: Box<dyn std::error::Error>,
        },
    }

    // define a complex struct
    fn complex_struct_def() -> proc_macro2::TokenStream {
        quote!(
            #[doc = "name-value-meta"]
            #[deprecated]
            #[allow(dead_code)]
            struct TestStruct<
                'a,
                'b: 'a,
                'c,
                T,
                #[cfg(not(target_os = "linux"))] E: Eq,
                #[cfg(with_len)] const LEN: usize,
            >
            where
                T: std::fmt::Debug + Clone + 'a + ?Sized,
                &'a T: Default,
                'c: 'a,
            {
                #[doc = "inner"]
                inner: Box<T>,
                #[cfg(not(target_os = "linux"))]
                pub member: &'a E,
                pub(crate) name: &'a mut str,
                #[cfg(unix)]
                pub(super) id: &'b i32,
                #[cfg_attr(target_os = "macos", deny(lint))]
                pub(in crate::tests) admin: &'c bool,
            }
        )
    }

    fn enum_def() -> proc_macro2::TokenStream {
        quote!(
            enum TestEnum {
                Good(Option<i32>),
                Bad {
                    reason: String,
                    trace: Box<dyn std::error::Error>,
                },
            }
        )
    }

    #[test]
    fn it_works() {
        let item: syn::ItemStruct = syn::parse2(complex_struct_def()).unwrap();
        let visitor = StructVisitor(&item);

        let metas = visitor.attrs();
        metas
            .get_list_by_ident("allow")
            .iter()
            .for_each(|item| println!("allow: {:?}", item));

        let item: syn::ItemEnum = syn::parse2(enum_def()).unwrap();

        dbg!(item);
    }
}
