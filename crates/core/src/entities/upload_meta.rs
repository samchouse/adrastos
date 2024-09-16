use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};

#[enum_def]
#[derive(Debug, Serialize, Deserialize, Clone, DbSelect, DbCommon, DbQuery)]
#[serde(rename_all = "camelCase")]
#[adrastos(rename = "upload_metadata")]
pub struct UploadMetadata {
    pub id: String,
    #[adrastos(find)]
    pub name: String,
    // #[adrastos(relation = User)] // TODO(@samchouse): undo this
    pub user_id: String,
    pub created_at: DateTime<Utc>,
}
