use crate::model::prelude::*;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
// make everything private to help enforce permissions!
pub struct PolicyDelegation {
    uuid: Uuid,
    _revision: i32,
    desc: String,
    from: Uuid,
    to: UuidObjectOption,
    resource: UuidObjectOption,
    action: String,
    granted_at: DateTime<Utc>,
    valid_until: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
// Used to apply changes (e.g. read patch request from API)
struct PolicyDelegationChange {
    pub uuid: Uuid,
    pub desc: Option<String>,
    pub from: Option<Uuid>,
    pub to: Option<UuidObjectOption>,
    pub resource: Option<UuidObjectOption>,
    pub action: Option<String>,
    pub granted_at: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}
