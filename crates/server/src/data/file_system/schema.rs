//! Schema of key-value storage based on file system and [`store::FsStore`]

use crate::ProblemID;

use super::{FileSysTable, SanitizeError, SanitizedString};
use problem::StandardProblem;
use store::Handle;

macro_rules! def_schema {
    ($( #[$attrs:meta] )* $name:ident <$life:lifetime>, $key:ty, $item:ty $(,)?) => {
        #[allow(non_camel_case_types)]
        $( #[$attrs] )*
        pub struct $name(Handle);

        impl $name {
            pub fn conn(ctx: &Handle) -> Self {
                Self(ctx.join(stringify!($name)))
            }
        }

        impl <$life> FileSysTable <$life> for $name {
            type Key = $key;
            type Item = $item;

            fn ctx(&self) -> &store::Handle {
                &self.0
            }
        }
    };
}

def_schema!(
    /// OJ problem data
    ojdata<'t>,
    &'t ProblemID,
    StandardProblem,
);

def_schema!(
    /// Global static data (e.g. legacy pdf files).
    global_staticdata<'t>,
    &'t str,
    std::fs::File,
);

def_schema!(
    /// Problem static data. if not found, one may fallback to the [`global_staticdata`]
    problem_staticdata<'t>,
    (&'t ProblemID, &'t str),
    std::fs::File,
);

impl TryFrom<(&ProblemID, &str)> for SanitizedString {
    type Error = SanitizeError;

    fn try_from((id, s): (&ProblemID, &str)) -> Result<Self, Self::Error> {
        SanitizedString::new(&format!("{id}/{s}"))
    }
}

impl TryFrom<&str> for SanitizedString {
    type Error = SanitizeError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SanitizedString::new(s)
    }
}
