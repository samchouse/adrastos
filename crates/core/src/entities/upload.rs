use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbSelect, DbCommon, DbQuery)]
pub struct Upload {
    pub id: String,
    #[adrastos(find)]
    pub name: String,
    pub created_at: DateTime<Utc>,
}
