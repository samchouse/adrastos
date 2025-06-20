use core::fmt;
use std::{fs::File, io::BufReader, sync::Arc};

use chrono::Duration;
use deadpool_postgres::{
    tokio_postgres::{Config, NoTls},
    Manager, ManagerConfig, Pool, RecyclingMethod,
};
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile::certs;
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_postgres::config::SslMode;
use tokio_postgres_rustls::MakeRustlsConnect;

use crate::{config, entities, expiring_map::ExpiringMap};

pub enum Error {
    UniqueKeyViolation,
    NonExistentTable,
}

impl TryFrom<&str> for Error {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            _ if value == Error::UniqueKeyViolation.to_string() => Ok(Error::UniqueKeyViolation),
            _ if value == Error::NonExistentTable.to_string() => Ok(Error::NonExistentTable),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Error::UniqueKeyViolation => "NewUniquenessConstraintViolationError",
            Error::NonExistentTable => "NewUndefinedRelationError",
        };

        write!(f, "{name}")
    }
}

fn create_pool(db_type: &DatabaseType, config: &config::Config) -> Pool {
    let mut pg_config = config.postgres_url.parse::<Config>().unwrap();
    pg_config.dbname(db_type.to_string().as_str());

    let manager_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };

    let mgr = match pg_config.get_ssl_mode() {
        SslMode::Disable => Manager::from_config(pg_config, NoTls, manager_config),
        _ => {
            let Some(certs_path) = &config.certs_path else {
                panic!("Certs path not set")
            };

            let ca_cert =
                &mut BufReader::new(File::open(format!("{}/cockroach.crt", certs_path)).unwrap());
            let ca_cert = certs(ca_cert).next().unwrap().unwrap();

            let mut root_store = RootCertStore::empty();
            root_store.add(ca_cert).unwrap();

            let config = ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            Manager::from_config(pg_config, MakeRustlsConnect::new(config), manager_config)
        }
    };

    Pool::builder(mgr).max_size(16).build().unwrap()
}

#[derive(Default)]
pub struct Databases(pub Arc<RwLock<ExpiringMap<DatabaseType, Pool>>>);

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum DatabaseType {
    System,
    Project(String),
}

impl fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseType::System => write!(f, "adrastos_system"),
            DatabaseType::Project(name) => write!(f, "adrastos_{}", name.to_lowercase()),
        }
    }
}

impl Databases {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ExpiringMap::new())))
    }

    pub async fn get(&self, db_type: &DatabaseType, config: &config::Config) -> Arc<Pool> {
        self.0
            .write()
            .await
            .reset_expiry(db_type, Duration::try_hours(1).unwrap());
        if let Some(pool) = self.0.read().await.get(db_type) {
            return pool.clone();
        }

        let pool = create_pool(db_type, config);
        pool.get()
            .await
            .unwrap()
            .execute(&format!("CREATE DATABASE IF NOT EXISTS {}", db_type), &[])
            .await
            .unwrap();
        entities::init(db_type, &pool, config).await;

        self.0.write().await.insert(
            db_type.clone(),
            pool.clone(),
            Duration::try_hours(1).unwrap(),
        );
        self.0.read().await.get(db_type).unwrap().clone()
    }

    pub async fn start_expiry_worker(databases: Arc<Databases>) -> JoinHandle<()> {
        ExpiringMap::start_expiry_worker(databases.0.clone(), tokio::time::Duration::from_mins(10))
            .await
    }
}

#[derive(Debug, Clone)]
pub struct Database(pub Arc<deadpool_postgres::Pool>, pub DatabaseType);

impl std::ops::Deref for Database {
    type Target = Arc<deadpool_postgres::Pool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
