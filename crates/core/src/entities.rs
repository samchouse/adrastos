use chrono::{DateTime, Utc};
use sea_query::enum_def;

#[enum_def]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub verified: bool,
    pub banned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
