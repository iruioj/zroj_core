use crate::serde_attr::*;

pub trait ProvideDefault<T> {
    fn provide_default(self, rec: T) -> T;
}

enum RenameOption {
    Specify(String),
    Format(RenameAllKind),
}

#[derive(Default)]
pub struct FieldContext {
    rename: Option<RenameOption>,
    pub flatten: bool,
    // skip or skip_serializing
    skip_serializing: bool,
    // 目前不支持，如果需要的话得手动指定 TsType
    pub serialize_with: bool,
    // 目前不支持，如果需要的话得手动指定 TsType
    pub with: bool,
    // 目前不支持，如果需要的话得手动指定 TsType
    pub getter: bool,
}

impl FieldContext {
    pub fn rename_field(&self, default: String) -> String {
        self.rename
            .as_ref()
            .map(|s| match s {
                RenameOption::Specify(s) => s.clone(),
                RenameOption::Format(f) => f.apply_to_field(&default),
            })
            .unwrap_or(default)
    }
    pub fn is_skip(&self) -> bool {
        self.skip_serializing
    }
    pub fn from_attr(attrs: Vec<FieldAttr>) -> Self {
        let mut r = FieldContext::default();
        for attr in attrs {
            match attr {
                FieldAttr::Rename(i) => match i {
                    SeperableMeta::Consistent(s) => r.rename = Some(RenameOption::Specify(s)),
                    SeperableMeta::Seperate { serialize, .. } => {
                        r.rename = serialize.map(RenameOption::Specify)
                    }
                },
                FieldAttr::Flatten => r.flatten = true,
                FieldAttr::Skip => r.skip_serializing = true,
                FieldAttr::SkipSerializing => r.skip_serializing = true,
                FieldAttr::SerializeWith(_) => r.serialize_with = true,
                FieldAttr::With(_) => r.with = true,
                FieldAttr::Getter(_) => r.getter = true,

                FieldAttr::SkipSerializingIf(_)
                | FieldAttr::Alias(_)
                | FieldAttr::Default(_)
                | FieldAttr::SkipDeserializing
                | FieldAttr::DeserializeWith(_)
                | FieldAttr::Bound(_)
                | FieldAttr::Borrow(_) => {}
            }
        }
        r
    }
}

#[derive(Default)]
pub struct VariantContext {
    rename: Option<RenameOption>,
    rename_all: Option<RenameAllKind>,
    // skip or skip_serializing
    skip_serializing: bool,
    // 目前不支持，如果需要的话得手动指定 TsType
    pub serialize_with: bool,
    // 目前不支持，如果需要的话得手动指定 TsType
    pub with: bool,
}

impl VariantContext {
    pub fn from_attr(attrs: Vec<VariantAttr>) -> Self {
        let mut r = VariantContext::default();
        for attr in attrs {
            match attr {
                VariantAttr::Rename(i) => match i {
                    SeperableMeta::Consistent(s) => r.rename = Some(RenameOption::Specify(s)),
                    SeperableMeta::Seperate { serialize, .. } => {
                        r.rename = serialize.map(RenameOption::Specify)
                    }
                },
                VariantAttr::RenameAll(i) => match i {
                    SeperableMeta::Consistent(s) => r.rename_all = Some(s),
                    SeperableMeta::Seperate { serialize, .. } => r.rename_all = serialize,
                },
                VariantAttr::Skip => r.skip_serializing = true,
                VariantAttr::SkipSerializing => r.skip_serializing = true,
                VariantAttr::SerializeWith(_) => r.serialize_with = true,
                VariantAttr::With(_) => r.with = true,

                VariantAttr::Alias(_)
                | VariantAttr::SkipDeserializing
                | VariantAttr::DeserializeWith(_)
                | VariantAttr::Bound(_)
                | VariantAttr::Borrow(_)
                | VariantAttr::Other => {}
            }
        }
        r
    }
    pub fn rename_variant(&self, default: String) -> String {
        self.rename
            .as_ref()
            .map(|s| match s {
                RenameOption::Specify(s) => s.clone(),
                RenameOption::Format(f) => f.apply_to_variant(&default),
            })
            .unwrap_or(default)
    }
    pub fn is_skip(&self) -> bool {
        self.skip_serializing
    }
}

#[derive(Default)]
pub struct ContainerContext {
    rename: Option<String>,
    rename_all: Option<RenameAllKind>,
    tag: Option<String>,
    content: Option<String>,
    untagged: bool,
    // 这东西的机制比较复杂，目前不支持
    pub remote: bool,
    pub transparent: bool,
    pub into: Option<String>,
}

impl ContainerContext {
    pub fn from_attr(attrs: Vec<ContainerAttr>) -> Self {
        let mut r = ContainerContext::default();
        for attr in attrs {
            match attr {
                ContainerAttr::Rename(i) => match i {
                    SeperableMeta::Consistent(s) => r.rename = Some(s),
                    SeperableMeta::Seperate { serialize, .. } => r.rename = serialize,
                },
                ContainerAttr::RenameAll(i) => match i {
                    SeperableMeta::Consistent(s) => r.rename_all = Some(s),
                    SeperableMeta::Seperate { serialize, .. } => r.rename_all = serialize,
                },
                ContainerAttr::Tag(s) => r.tag = Some(s),
                ContainerAttr::Content(s) => r.content = Some(s),
                ContainerAttr::Untagged => r.untagged = true,
                ContainerAttr::Remote(_) => r.remote = true,
                ContainerAttr::Transparent => r.transparent = true,
                ContainerAttr::Into(s) => r.into = Some(s),

                ContainerAttr::DenyUnknownFields
                | ContainerAttr::Bound(_)
                | ContainerAttr::Default(_)
                | ContainerAttr::From(_)
                | ContainerAttr::TryFrom(_)
                | ContainerAttr::Crate(_) => {}
            }
        }
        r
    }

    pub fn rename(&self, default: String) -> String {
        self.rename.clone().unwrap_or(default)
    }
    pub fn tag(&self) -> Option<String> {
        self.tag.clone()
    }
    pub fn content_tag(&self) -> Option<String> {
        self.content.clone()
    }
    pub fn untagged(&self) -> bool {
        self.untagged
    }
}

impl ProvideDefault<FieldContext> for &'_ ContainerContext {
    fn provide_default(self, mut rec: FieldContext) -> FieldContext {
        if rec.rename.is_none() {
            if let Some(rule) = &self.rename_all {
                rec.rename = Some(RenameOption::Format(rule.clone()));
            }
        }
        rec
    }
}

impl ProvideDefault<VariantContext> for &'_ ContainerContext {
    fn provide_default(self, mut rec: VariantContext) -> VariantContext {
        if rec.rename.is_none() {
            if let Some(rule) = &self.rename_all {
                rec.rename = Some(RenameOption::Format(rule.clone()));
            }
        }
        rec
    }
}

impl ProvideDefault<FieldContext> for &'_ VariantContext {
    fn provide_default(self, mut rec: FieldContext) -> FieldContext {
        if rec.rename.is_none() {
            if let Some(rule) = &self.rename_all {
                rec.rename = Some(RenameOption::Format(rule.clone()));
            }
        }
        rec
    }
}
