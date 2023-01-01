use super::User;

pub async fn migrate(db_pool: deadpool_postgres::Pool) {
    let mutations = vec![User::migrate()];

    for mutation in mutations {
        let conn = db_pool.get().await.unwrap();
        conn.execute(mutation.as_str(), &[]).await.unwrap();
    }
}
