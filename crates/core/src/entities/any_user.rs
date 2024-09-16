use chrono::{DateTime, Utc};
use sea_query::enum_def;
use serde::{Deserialize, Serialize};

use crate::db::postgres::{Database, DatabaseType};

use super::{
    Connection, Passkey, RefreshTokenTree, SystemUser, UpdateSystemUser, UpdateUser, User,
};

#[derive(Debug, Clone)]
pub enum UserType<'a> {
    System(&'a Database),
    Normal(&'a Database),
}

#[derive(Debug, Clone)]
pub enum AlternateUserType {
    System,
    Normal,
}

#[enum_def]
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct AnyUser {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing)]
    pub mfa_secret: Option<String>,
    #[serde(skip_serializing)]
    pub mfa_backup_codes: Option<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing)]
    pub connections: Option<Vec<Connection>>,
    #[serde(skip_serializing)]
    pub refresh_token_trees: Option<Vec<RefreshTokenTree>>,
    #[serde(skip_serializing)]
    pub passkeys: Option<Vec<Passkey>>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateAnyUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub mfa_secret: Option<Option<String>>,
    pub mfa_backup_codes: Option<Option<Vec<String>>>,
}

#[derive(Debug, Clone)]
pub struct AnyUserSelectBuilder<'a> {
    db: &'a Database,
    user_type: UserType<'a>,
    query_builder: sea_query::SelectStatement,
}

impl<'a> From<&'a Database> for UserType<'a> {
    fn from(value: &'a Database) -> Self {
        match value.1 {
            DatabaseType::System => UserType::System(value),
            DatabaseType::Project(_) => UserType::Normal(value),
        }
    }
}

impl<'a> UserType<'a> {
    fn table(&self) -> sea_query::Alias {
        match self {
            UserType::System(_) => SystemUser::table(),
            UserType::Normal(_) => User::table(),
        }
    }

    fn error_identifier(&self) -> String {
        match self {
            UserType::System(_) => User::error_identifier(),
            UserType::Normal(_) => SystemUser::error_identifier(),
        }
        .to_string()
    }

    pub async fn create(&self, user: AnyUser) -> Result<(), crate::error::Error> {
        match self {
            UserType::System(db) => {
                SystemUser {
                    id: user.id,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    email: user.email,
                    username: user.username,
                    password: user.password,
                    created_at: user.created_at,
                    ..Default::default()
                }
                .create(db)
                .await
            }
            UserType::Normal(db) => {
                User {
                    id: user.id,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    email: user.email,
                    username: user.username,
                    password: user.password,
                    created_at: user.created_at,
                    ..Default::default()
                }
                .create(db)
                .await
            }
        }
    }

    pub async fn update(
        &self,
        user: AnyUser,
        update: UpdateAnyUser,
    ) -> Result<(), crate::error::Error> {
        match self {
            UserType::System(db) => {
                SystemUser {
                    id: user.id,
                    ..Default::default()
                }
                .update(
                    db,
                    UpdateSystemUser {
                        first_name: update.first_name,
                        last_name: update.last_name,
                        email: update.email,
                        username: update.username,
                        password: update.password,
                        mfa_secret: update.mfa_secret,
                        mfa_backup_codes: update.mfa_backup_codes,
                    },
                )
                .await
            }
            UserType::Normal(db) => {
                User {
                    id: user.id,
                    ..Default::default()
                }
                .update(
                    db,
                    UpdateUser {
                        first_name: update.first_name,
                        last_name: update.last_name,
                        email: update.email,
                        username: update.username,
                        password: update.password,
                        mfa_secret: update.mfa_secret,
                        mfa_backup_codes: update.mfa_backup_codes,
                        ..Default::default()
                    },
                )
                .await
            }
        }
    }

    pub async fn delete(&self, user: AnyUser) -> Result<(), crate::error::Error> {
        match self {
            UserType::System(db) => {
                SystemUser {
                    id: user.id,
                    ..Default::default()
                }
                .delete(db)
                .await
            }
            UserType::Normal(db) => {
                User {
                    id: user.id,
                    ..Default::default()
                }
                .delete(db)
                .await
            }
        }
    }

    pub fn find(&self) -> AnyUserSelectBuilder {
        AnyUserSelectBuilder {
            user_type: self.clone(),
            db: match self {
                UserType::System(db) => db,
                UserType::Normal(db) => db,
            },
            query_builder: sea_query::Query::select()
                .from(Self::table(self))
                .columns([
                    AnyUserIden::Id,
                    AnyUserIden::FirstName,
                    AnyUserIden::LastName,
                    AnyUserIden::Email,
                    AnyUserIden::Username,
                    AnyUserIden::Password,
                    AnyUserIden::MfaSecret,
                    AnyUserIden::MfaBackupCodes,
                    AnyUserIden::CreatedAt,
                    AnyUserIden::UpdatedAt,
                ])
                .to_owned(),
        }
    }

    pub fn find_by_id(&self, id: &str) -> AnyUserSelectBuilder {
        let mut builder = Self::find(self);
        builder.by_id(id).to_owned()
    }
}

pub enum AnyUserJoin {
    Connections,
    RefreshTokenTrees,
    Passkeys,
}

impl AnyUserSelectBuilder<'_> {
    fn by_id(&mut self, id: &str) -> &mut Self {
        self.query_builder
            .and_where(sea_query::Expr::col(AnyUserIden::Id).eq(id));

        self
    }

    async fn finish(
        &mut self,
        db: &deadpool_postgres::Pool,
    ) -> Result<Vec<deadpool_postgres::tokio_postgres::Row>, crate::error::Error> {
        let rows = db
            .get()
            .await
            .unwrap()
            .query(self.to_string().as_str(), &[])
            .await
            .map_err(|e| {
                let error = format!(
                    "An error occurred while fetching the {}: {e}",
                    self.user_type.error_identifier(),
                );
                crate::error::Error::InternalServerError(error)
            })?;

        Ok(rows)
    }

    pub fn by_email(&mut self, email: String) -> &mut Self {
        self.query_builder
            .and_where(sea_query::Expr::col(AnyUserIden::Email).eq(email));

        self
    }

    pub fn by_username(&mut self, username: String) -> &mut Self {
        self.query_builder
            .and_where(sea_query::Expr::col(AnyUserIden::Username).eq(username));

        self
    }

    pub fn and_where(&mut self, expressions: Vec<sea_query::SimpleExpr>) -> &mut Self {
        for expression in expressions {
            self.query_builder.and_where(expression);
        }

        self
    }

    pub fn join(&mut self, join: AnyUserJoin) -> &mut Self {
        let query = match join {
            AnyUserJoin::Connections => Connection::find()
                .and_where(vec![sea_query::Expr::col(sea_query::Alias::new("user_id"))
                    .equals((Connection::table(), sea_query::Alias::new("user_id")))])
                .to_string(),
            AnyUserJoin::RefreshTokenTrees => RefreshTokenTree::find()
                .and_where(vec![sea_query::Expr::col(sea_query::Alias::new("user_id"))
                    .equals((
                        RefreshTokenTree::table(),
                        sea_query::Alias::new("user_id"),
                    ))])
                .to_string(),
            AnyUserJoin::Passkeys => Passkey::find()
                .and_where(vec![sea_query::Expr::col(sea_query::Alias::new("user_id"))
                    .equals((Passkey::table(), sea_query::Alias::new("user_id")))])
                .to_string(),
        };

        self.query_builder.expr(sea_query::Expr::cust(
            format!("(SELECT json_agg({join}) FROM ({query}) {join}) as {join}",).as_str(),
        ));

        self
    }

    pub async fn one(&mut self) -> Result<AnyUser, crate::error::Error> {
        self.query_builder.reset_limit().limit(1);

        Ok(self
            .finish(self.db)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| {
                let message = format!("No {} was found", self.user_type.error_identifier());
                crate::error::Error::BadRequest(message)
            })?
            .into())
    }

    pub async fn all(&mut self) -> Result<Vec<AnyUser>, crate::error::Error> {
        self.query_builder.reset_limit();

        // TODO(@samchouse): add pagination, etc.
        Ok(self
            .finish(self.db)
            .await?
            .into_iter()
            .map(|row| row.into())
            .collect::<Vec<_>>())
    }
}

impl std::fmt::Display for AnyUserSelectBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.query_builder
                .to_string(sea_query::PostgresQueryBuilder)
        )
    }
}

impl From<deadpool_postgres::tokio_postgres::Row> for AnyUser {
    fn from(row: deadpool_postgres::tokio_postgres::Row) -> AnyUser {
        AnyUser {
            id: row.get("id"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            email: row.get("email"),
            username: row.get("username"),
            password: row.get("password"),
            mfa_secret: row.get("mfa_secret"),
            mfa_backup_codes: row.get("mfa_backup_codes"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),

            connections: row
                .try_get::<_, serde_json::Value>("connections")
                .ok()
                .map(|v| serde_json::from_value(v).unwrap()),
            refresh_token_trees: row
                .try_get::<_, serde_json::Value>("refresh_token_trees")
                .ok()
                .map(|v| serde_json::from_value(v).unwrap()),
            passkeys: row
                .try_get::<_, serde_json::Value>("passkeys")
                .ok()
                .map(|v| serde_json::from_value(v).unwrap()),
        }
    }
}

impl std::fmt::Display for AnyUserJoin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            AnyUserJoin::Connections => "connections",
            AnyUserJoin::RefreshTokenTrees => "refresh_token_trees",
            AnyUserJoin::Passkeys => "passkeys",
        };

        write!(f, "{}", name)
    }
}

impl From<User> for AnyUser {
    fn from(value: User) -> Self {
        AnyUser {
            id: value.id,
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            username: value.username,
            password: value.password,
            created_at: value.created_at,
            updated_at: value.updated_at,
            mfa_secret: value.mfa_secret,
            mfa_backup_codes: value.mfa_backup_codes,

            connections: value.connections,
            refresh_token_trees: value.refresh_token_trees,
            passkeys: value.passkeys,
        }
    }
}

impl From<SystemUser> for AnyUser {
    fn from(value: SystemUser) -> Self {
        AnyUser {
            id: value.id,
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            username: value.username,
            password: value.password,
            created_at: value.created_at,
            updated_at: value.updated_at,
            mfa_secret: value.mfa_secret,
            mfa_backup_codes: value.mfa_backup_codes,

            connections: value.connections,
            refresh_token_trees: value.refresh_token_trees,
            passkeys: value.passkeys,
        }
    }
}
