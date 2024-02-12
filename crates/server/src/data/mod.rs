//! Database backends and data schemas used across the application.
//!
//! Several database API is defined here, along with some data types. These types
//! are not necessarily serializable, but must be used in this submodule.

pub mod error;
pub mod file_system;
pub mod mysql;
pub mod types;

// database
pub mod problem_ojdata;
pub mod problem_statement;
pub mod submission;
pub mod user;
