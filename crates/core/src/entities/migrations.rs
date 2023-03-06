use super::{custom_table::schema::CustomTableSchema, Connection, Migrate, RefreshTokenTree, User};

pub async fn migrate(db_pool: &deadpool_postgres::Pool) {
    let conn = db_pool.get().await.unwrap();
    let migrations = vec![
        User::migrate(),
        Connection::migrate(),
        RefreshTokenTree::migrate(),
        CustomTableSchema::migrate(),
    ];

    for migration in migrations {
        conn.execute(migration.as_str(), &[]).await.unwrap();
    }
}
