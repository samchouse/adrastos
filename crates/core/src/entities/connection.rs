use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};

use super::User;

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbSelect, DbCommon, DbQuery)]
pub struct Connection {
    pub id: String,
    #[adrastos(find)]
    pub provider: String,
    #[adrastos(relation = User)]
    pub user_id: String,
    #[adrastos(find)]
    pub provider_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
