use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};

use super::Team;

#[enum_def]
#[derive(Debug, Default, Serialize, Deserialize, Clone, DbSelect, DbCommon, DbQuery)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Project {
    pub id: String,
    #[adrastos(unique)]
    pub name: String,
    pub hostnames: Vec<String>,
    #[adrastos(relation = Team)]
    pub team_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
