fn type_param_filter(param: &syn::GenericParam) -> Option<&syn::TypeParam> {
    match param {
        syn::GenericParam::Type(ty) => Some(ty),
        _ => None,
    }
}

fn type_predict_filter(predict: &syn::WherePredicate) -> Option<&syn::PredicateType> {
    match predict {
        syn::WherePredicate::Type(ty) => Some(ty),
        _ => None,
    }
}

pub struct GenericsVisitor<'v>(pub &'v syn::Generics);

impl<'v> GenericsVisitor<'v> {
    fn iter_params(&self) -> syn::punctuated::Iter<'_, syn::GenericParam> {
        self.0.params.iter()
    }
    pub fn iter_where_clauses(&self) -> Option<syn::punctuated::Iter<'_, syn::WherePredicate>> {
        self.0.where_clause.as_ref().map(|c| c.predicates.iter())
    }
    pub fn iter_type_params(
        &self,
    ) -> std::iter::FilterMap<
        syn::punctuated::Iter<'_, syn::GenericParam>,
        fn(&syn::GenericParam) -> Option<&syn::TypeParam>,
    > {
        self.iter_params().filter_map(type_param_filter)
    }
    #[allow(clippy::manual_map)]
    pub fn iter_type_where_clauses(
        &self,
    ) -> Option<
        std::iter::FilterMap<
            syn::punctuated::Iter<'_, syn::WherePredicate>,
            fn(&syn::WherePredicate) -> Option<&syn::PredicateType>,
        >,
    > {
        if let Some(iter) = self.iter_where_clauses() {
            Some(iter.filter_map(type_predict_filter))
        } else {
            None
        }
    }
    /// transform `<'b: 'a, T: Debug, R>` to `<'b, T, R>`.
    pub fn get_pure_params(
        &self,
    ) -> syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma> {
        let mut r = self.0.params.clone();
        r.iter_mut().for_each(|o| match o {
            syn::GenericParam::Type(o) => {
                o.bounds.clear();
            }
            syn::GenericParam::Lifetime(o) => {
                o.bounds.clear();
            }
            syn::GenericParam::Const(_) => unimplemented!(),
        });
        r
    }
}
