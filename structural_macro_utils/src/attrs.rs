struct MetaList(Vec<syn::Meta>);
impl syn::parse::Parse for MetaList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let metas = input.parse_terminated(syn::Meta::parse, syn::Token![,])?;
        Ok(MetaList(metas.into_iter().collect()))
    }
}

fn outer_style_filter(o: &syn::Attribute) -> Option<&syn::Meta> {
    match o.style {
        syn::AttrStyle::Outer => Some(&o.meta),
        syn::AttrStyle::Inner(_) => None,
    }
}
fn name_value_meta_filter(o: &syn::Meta) -> Option<&syn::MetaNameValue> {
    match o {
        syn::Meta::NameValue(o) => Some(o),
        _ => None,
    }
}
fn path_meta_filter(o: &syn::Meta) -> Option<&syn::Path> {
    match o {
        syn::Meta::Path(o) => Some(o),
        _ => None,
    }
}

pub struct MetaListVisitor(Vec<syn::Meta>);

impl MetaListVisitor {
    pub fn iter(&self) -> std::slice::Iter<'_, syn::Meta> {
        self.0.iter()
    }
    pub fn iter_name_values(
        &self,
    ) -> std::iter::FilterMap<
        std::slice::Iter<'_, syn::Meta>,
        fn(&syn::Meta) -> Option<&syn::MetaNameValue>,
    > {
        self.0.iter().filter_map(name_value_meta_filter)
    }
    pub fn iter_path(
        &self,
    ) -> std::iter::FilterMap<std::slice::Iter<'_, syn::Meta>, fn(&syn::Meta) -> Option<&syn::Path>>
    {
        self.0.iter().filter_map(path_meta_filter)
    }
    pub fn get_list_by_ident(&self, path_ident: &str) -> Vec<syn::Meta> {
        self.0
            .iter()
            .filter_map(|o| match o {
                syn::Meta::List(o) => {
                    if o.path.is_ident(path_ident) {
                        let metas: MetaList = syn::parse2(o.tokens.clone()).unwrap();
                        Some(metas.0)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .reduce(|mut a, b| {
                a.extend(b);
                a
            })
            .unwrap_or_default()
    }
}

pub struct AttrListVisitor<'v>(pub &'v [syn::Attribute]);

impl<'v> AttrListVisitor<'v> {
    pub fn iter_metas(
        &self,
    ) -> std::iter::FilterMap<
        std::slice::Iter<'_, syn::Attribute>,
        fn(&syn::Attribute) -> Option<&syn::Meta>,
    > {
        self.0.iter().filter_map(outer_style_filter)
    }
    pub fn iter_name_values(
        &self,
    ) -> std::iter::FilterMap<
        std::iter::FilterMap<
            std::slice::Iter<'_, syn::Attribute>,
            fn(&syn::Attribute) -> Option<&syn::Meta>,
        >,
        fn(&syn::Meta) -> Option<&syn::MetaNameValue>,
    > {
        self.iter_metas().filter_map(name_value_meta_filter)
    }
    pub fn iter_path(
        &self,
    ) -> std::iter::FilterMap<
        std::iter::FilterMap<
            std::slice::Iter<'_, syn::Attribute>,
            fn(&syn::Attribute) -> Option<&syn::Meta>,
        >,
        fn(&syn::Meta) -> Option<&syn::Path>,
    > {
        self.iter_metas().filter_map(path_meta_filter)
    }
    pub fn get_list_by_ident(&self, path_ident: &str) -> MetaListVisitor {
        MetaListVisitor(
            self.iter_metas()
                .filter_map(|o| match o {
                    syn::Meta::List(o) => {
                        if o.path.is_ident(path_ident) {
                            let metas: MetaList = syn::parse2(o.tokens.clone()).unwrap();
                            Some(metas.0)
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
                .reduce(|mut a, b| {
                    a.extend(b);
                    a
                })
                .unwrap_or_default(),
        )
    }
}
