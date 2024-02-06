use crate::ProblemID;

use super::{FileSysTable, SanitizedString};
use problem::StandardProblem;
use store::Handle;

macro_rules! def_schema {
    ($name:ident, $key:ty, $item:ty) => {
        #[allow(non_camel_case_types)]
        pub struct $name(Handle);

        impl $name {
            pub fn conn(ctx: &Handle) -> Self {
                Self(ctx.join(stringify!($name)))
            }
        }

        impl FileSysTable for $name {
            type Key = $key;
            type Item = $item;

            fn ctx(&self) -> &store::Handle {
                &self.0
            }
        }
    };
}

def_schema!(ojdata, ProblemID, StandardProblem);
def_schema!(staticdata, SanitizedString, std::fs::File);
