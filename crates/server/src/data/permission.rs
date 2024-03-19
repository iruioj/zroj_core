//! Implement a RBAC model for permission control.
//!
//! Firstly, a user (subject) possesses a series of roles (groupuser, contestant,
//! admin, problem owner, etc.). A role does not have a parent role.
//!
//! A resource (object) is basically a publicly exposed API method of a struct
//! in [`crate::data`]. Each resource (problem r/w, contest r/w, etc.) is marked
//! by a string id, dependent with the query parameters. For simplicity, we do not
//! provide roles/groups for resources.
//!
//! Note that since the number of contests can grow fast, it is preferred to treat
//! contestant as a subject instead of a role.
//!
//! This model is designed for **time-independent** permission. For time-varying permissions
//! please implement your logic in resource handler.
//!
//! The detailed model can be written as
//!
//! ```no_run rust
//! fn query(sub, obj) {
//!     let sub_roles = roles_of(sub); /* often < 100 */
//!     let obj_roles = accessible_roles_of(obj); /* often < 10 */
//!     for role in obj_roles {
//!         if sub_roles.contains(role) {
//!             return true
//!         }
//!     }
//!     return false
//! }
//! ```

use std::{collections::HashMap, sync::RwLock};

use smallvec::SmallVec;

use crate::{data::error::DataError, UserID};

pub const ROOT_USER_ID: UserID = 1;

struct RoleSet(SmallVec<[u64; 10]>);

struct PermissonManagerInner {
    roles_of_user: HashMap<UserID, RoleSet>,
    access_roles_of_res: HashMap<u64, RoleSet>,
}
pub struct PermissionManager(RwLock<PermissonManagerInner>);

impl PermissionManager {
    pub fn new() -> Self {
        Self(RwLock::new(PermissonManagerInner {
            roles_of_user: Default::default(),
            access_roles_of_res: Default::default(),
        }))
    }
    pub fn query<T: Resource>(
        &self,
        user_id: UserID,
        r: ResourceHandle<T>,
    ) -> Result<T::Item, DataError> {
        // bypass all checking for root
        if user_id == ROOT_USER_ID {
            return r.0.load();
        }

        let perm_id = r.0.perm_id();
        let state = self.0.read()?;
        let Some(roles) = state.roles_of_user.get(&user_id) else {
            return Err(DataError::Perm(user_id, perm_id));
        };
        let Some(res_roles) = state.access_roles_of_res.get(&r.0.perm_id()) else {
            return Err(DataError::Perm(user_id, perm_id));
        };
        for role in &res_roles.0 {
            if roles.0.contains(role) {
                return r.0.load();
            }
        }
        Err(DataError::Perm(user_id, perm_id))
    }
}

pub trait Resource {
    /// Type of the resource
    type Item;

    /// Global unique, time-invariant ID of the resource, for permission querying.
    /// Note that different resources are allowed to associate to the same ID
    /// (e.g. pagination).
    fn perm_id(&self) -> u64;

    /// load resource from database
    fn load(&self) -> Result<Self::Item, DataError>;
}

/// [`Resource`] is a public trait, and we need a wrapper struct to protect it.
pub struct ResourceHandle<T: Resource>(T);
impl<T: Resource> ResourceHandle<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}
