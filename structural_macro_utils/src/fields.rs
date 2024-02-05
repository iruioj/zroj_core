use crate::attrs::AttrListVisitor;

pub struct FieldIter<'b>(
    Option<
        std::iter::Map<
            syn::punctuated::Iter<'b, syn::Field>,
            fn(&'b syn::Field) -> FieldVisitor<'b>,
        >,
    >,
);

impl<'b> Iterator for FieldIter<'b> {
    type Item = FieldVisitor<'b>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Some(iter) => iter.next(),
            None => None,
        }
    }
}

pub struct FieldsVisitor<'v>(pub &'v syn::Fields);

impl<'v> FieldsVisitor<'v> {
    pub fn iter_fields<'b>(
        &'b self,
    ) -> std::iter::Map<syn::punctuated::Iter<'b, syn::Field>, fn(&'b syn::Field) -> FieldVisitor<'b>>
    where
        'v: 'b,
    {
        self.0.iter().map(FieldVisitor)
    }
    pub fn is_named(&self) -> bool {
        matches!(self.0, syn::Fields::Named(_))
    }
    pub fn is_unnamed(&self) -> bool {
        matches!(self.0, syn::Fields::Unnamed(_))
    }
    pub fn is_unit(&self) -> bool {
        matches!(self.0, syn::Fields::Unit)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn the_only_field(&self) -> Option<FieldVisitor> {
        let mut it = self.iter_fields();
        if it.len() != 1 {
            None
        } else {
            Some(it.next().unwrap())
        }
    }
}

pub struct FieldVisitor<'v>(pub &'v syn::Field);

impl<'v> FieldVisitor<'v> {
    pub fn attrs(&self) -> AttrListVisitor<'_> {
        AttrListVisitor(&self.0.attrs)
    }
    pub fn ident(&self) -> Option<&syn::Ident> {
        if let Some(ident) = &self.0.ident {
            Some(ident)
        } else {
            None
        }
    }
    pub fn ty(&self) -> &syn::Type {
        &self.0.ty
    }
}
