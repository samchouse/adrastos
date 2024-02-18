use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};

use super::Project;

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbSelect, DbCommon, DbQuery, Default)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Team {
    pub id: String,
    #[adrastos(unique)]
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,

    #[adrastos(join)]
    pub projects: Option<Vec<Project>>,
}
