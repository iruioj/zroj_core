use crate::{GroupID, UserID};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ProblemAccess {
    /// None
    None = 0,
    /// view problem and test data structure, real data if config.open_data
    View = 1,
    /// view any, edit general
    Edit = 2,
    /// view and edit any, or delete a problem
    Admin = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserGroup {
    User(UserID),
    Group(GroupID),
}

/// problem config, stored in config_path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemConfig {
    pub owner: UserID,
    pub access: Vec<(UserGroup, ProblemAccess)>,
    pub open_source_data: bool,
    pub create_date: String,
}
