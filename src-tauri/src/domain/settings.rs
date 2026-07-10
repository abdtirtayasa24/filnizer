use serde::{Deserialize, Serialize};

use crate::domain::operations::ConflictPolicy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_conflict_policy: ConflictPolicy,
    pub history_retention_days: Option<u32>,
    pub show_privacy_note: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_conflict_policy: ConflictPolicy::Rename,
            history_retention_days: Some(90),
            show_privacy_note: true,
        }
    }
}
