use super::{Connection, Migrate, RefreshTokenTree, User};

pub async fn migrate(db_pool: &deadpool_postgres::Pool) {
    let migrations = vec![
        User::migrate(),
        Connection::migrate(),
        RefreshTokenTree::migrate(),
    ];

    for migration in migrations {
        let conn = db_pool.get().await.unwrap();
        conn.execute(migration.as_str(), &[]).await.unwrap();
    }
}
