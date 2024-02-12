use adrastos_macros::{DbCommon, DbQuery, DbSelect};
use chrono::{DateTime, Utc};
use sea_query::{enum_def, Alias, Expr, PostgresQueryBuilder};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing_unwrap::ResultExt;
use validator::Validate;

use crate::{auth, error::Error};

use super::{Connection, RefreshTokenTree, Update};

fn validate_password(password: String) -> Result<String, Error> {
    auth::hash_password(&password).map_err(|err| {
        Error::InternalServerError(format!(
            "An error occurred while hashing the password for the {err}"
        ))
    })
}

#[enum_def]
#[derive(Debug, Default, Validate, Serialize, Deserialize, Clone, DbCommon, DbSelect, DbQuery)]
#[adrastos(validated)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    #[adrastos(find)]
    #[validate(length(max = 50))]
    pub first_name: String,
    #[adrastos(find)]
    #[validate(length(max = 50))]
    pub last_name: String,
    #[adrastos(find, unique)]
    #[validate(email)]
    pub email: String,
    #[adrastos(find, unique)]
    #[validate(length(min = 5, max = 64))]
    pub username: String,
    #[serde(skip_serializing)]
    #[validate(length(min = 8, max = 64))]
    #[adrastos(transform = validate_password)]
    pub password: String,
    pub verified: bool,
    pub banned: bool,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
    #[serde(skip_serializing)]
    pub mfa_backup_codes: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,

    #[adrastos(join)]
    #[serde(skip_serializing)]
    pub connections: Option<Vec<Connection>>,
    #[adrastos(join)]
    #[serde(skip_serializing)]
    pub refresh_token_trees: Option<Vec<RefreshTokenTree>>,
}

#[derive(Debug, Validate, Clone, Default)]
pub struct UpdateUser {
    #[validate(length(max = 50))]
    pub first_name: Option<String>,
    #[validate(length(max = 50))]
    pub last_name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 5, max = 64))]
    pub username: Option<String>,
    #[validate(length(min = 8, max = 64))]
    pub password: Option<String>,
    pub verified: Option<bool>,
    pub banned: Option<bool>,
    pub mfa_secret: Option<Option<String>>,
    pub mfa_backup_codes: Option<Option<Vec<String>>>,
}

impl User {
    pub async fn update(
        &self,
        db_pool: &deadpool_postgres::Pool,
        update: UpdateUser,
    ) -> Result<(), Error> {
        update.validate().map_err(|e| Error::ValidationErrors {
            errors: e,
            message: "Invalid user update".into(),
        })?;

        let query = sea_query::Query::update()
            .table(Self::table())
            .values(Update::create([
                (UserIden::FirstName, update.first_name.into()),
                (UserIden::LastName, update.last_name.into()),
                (UserIden::Email, update.email.into()),
                (UserIden::Username, update.username.into()),
                (
                    UserIden::Password,
                    update
                        .password
                        .map(|v| auth::hash_password(v.as_str()).unwrap_or_log())
                        .into(),
                ),
                (UserIden::Verified, update.verified.into()),
                (UserIden::Banned, update.banned.into()),
                (UserIden::MfaSecret, update.mfa_secret.into()),
                (UserIden::MfaBackupCodes, update.mfa_backup_codes.into()),
                (UserIden::UpdatedAt, Some(Utc::now()).into()),
            ]))
            .and_where(Expr::col(UserIden::Id).eq(self.id.clone()))
            .to_string(PostgresQueryBuilder);

        db_pool
            .get()
            .await
            .unwrap_or_log()
            .execute(&query, &[])
            .await
            .map_err(|e| {
                error!(error = ?e);
                Error::InternalServerError("Failed to update user".into())
            })?;

        Ok(())
    }
}
