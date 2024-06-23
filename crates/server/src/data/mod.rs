//! Database backends and data schemas used across the application.
//!
//! Several database API is defined here, along with some data types. These types
//! are not necessarily serializable, but must be used in this submodule.
//! Conventionally, publicly (in crate) exposed database APIs are not protected
//! by permission system.

pub mod error;
pub mod file_system;
pub mod mysql;
pub mod types;
pub mod databases;


// permission
mod permission;
pub use permission::{PermissionManager, Resource, ResourceHandle, ROOT_USER_ID};
